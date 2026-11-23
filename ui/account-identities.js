// Display and matching helpers only. Credentials are owned by provider CLIs;
// no account switch or login mutation takes place in the frontend.
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root && root.document) root.QhudAccountIdentities = api;
})(typeof window === "undefined" ? null : window, function () {
  "use strict";
  const text = (value) => typeof value === "string" ? value.trim() : "";
  const email = (value) => text(value).toLowerCase();
  const workspaceKey = (workspace) => JSON.stringify([email(workspace?.email), text(workspace?.account_id)]);
  const accountKey = (account) => JSON.stringify([
    email(account?.email), text(account?.account_id), text(account?.org_id),
    !account?.email && !account?.account_id ? text(account?.config_dir) || "default" : "",
  ]);

  function distinctWorkspaces(workspaces) {
    const unique = new Map();
    for (const row of workspaces || []) {
      const key = workspaceKey(row);
      if (!unique.has(key)) unique.set(key, row);
    }
    return [...unique.values()];
  }

  function activeWorkspace(workspaces, account, knownAccounts = []) {
    let candidates = distinctWorkspaces(workspaces).filter((row) => row.active);
    const id = text(account?.account_id);
    if (id) candidates = candidates.filter((row) => row.account_id === id);
    const expectedEmail = email(account?.email);
    if (!expectedEmail) {
      // The active flag belongs to a workspace, not to every unlabelled
      // pane. An ID alone is sufficient only when it has a single owner.
      if (!id) return null;
      const owners = new Set([...(workspaces || []), ...knownAccounts]
        .filter((other) => other?.account_id === id)
        .map((other) => email(other.email)).filter(Boolean));
      if (owners.size > 1) return null;
    }
    if (expectedEmail) {
      const exact = candidates.filter((row) => email(row.email) === expectedEmail);
      if (exact.length === 1) return exact[0];
      if (exact.length > 1) return null;
      // Compatibility for an older single-login snapshot, without assigning
      // an unknown-email result across two known, different logins.
      if (candidates.some((row) => email(row.email))) return null;
      const knownEmails = new Set(knownAccounts
        .filter((other) => !id || other?.account_id === id)
        .map((other) => email(other?.email)).filter(Boolean));
      if (knownEmails.size > 1) return null;
    }
    return candidates.length === 1 ? candidates[0] : null;
  }

  function claudeUsageForAccount(results, account) {
    return (Array.isArray(results) ? results : []).find((result) =>
      result.key === (account?.config_dir || "default") &&
      text(result.account_id) && result.account_id === account?.account_id &&
      text(result.org_id) === text(account?.org_id),
    ) || null;
  }

  function antigravityUsageForAccount(usage, row) {
    const owner = text(row?.account?.account_id) || text(row?.account?.email);
    if (!owner || text(usage?.account_id) !== owner) return null;
    const rowTime = Number.isFinite(row?.cache_fetched_at_ms) ? row.cache_fetched_at_ms : 0;
    const usageTime = Number.isFinite(usage?.fetched_at_ms) ? usage.fetched_at_ms : 0;
    // A newer CLI/diagnostic fetch can reach the next poll while this window
    // still remembers an older successful button click. The newer source wins.
    if (rowTime > 0 && usageTime < rowTime) return null;
    return usage;
  }

  function connectionsForAccount(profiles, provider, account) {
    const id = text(account?.account_id);
    const login = email(account?.email);
    if (!id || !login) return [];
    return (profiles || []).filter((profile) => profile.has_row && profile.provider === provider &&
      text(profile.account_id) === id && email(profile.email) === login &&
      (provider !== "claude" || (text(account?.org_id) && text(profile.org_id) === text(account.org_id))));
  }

  // A dated observation from the admin Members page is separate from the
  // provider's usage plan and from whether qhud can authenticate this profile.
  function seatAnnotationForProfile(profile, now = Date.now()) {
    if (profile?.provider !== "codex" || profile.scope !== "team") return null;
    const annotation = profile.seat_annotation;
    if (!annotation || !["standard", "premium"].includes(annotation.seat_type)) return null;
    const safeText = (value, max) => typeof value === "string" && value.trim().length > 0 &&
      value.length <= max && !/[\x00-\x1f\x7f]/.test(value);
    if (!safeText(annotation.email, 254) || !safeText(annotation.workspace_id, 128) ||
        !safeText(annotation.workspace_name, 160)) return null;
    if (!email(profile.email) || email(annotation.email) !== email(profile.email)) return null;
    if (!Number.isSafeInteger(annotation.verified_at_ms) || annotation.verified_at_ms <= 0 ||
        annotation.verified_at_ms > now + 60_000) return null;
    return { ...annotation, email: email(annotation.email), workspace_id: text(annotation.workspace_id), workspace_name: text(annotation.workspace_name) };
  }

  function seatAnnotationForAccount(profiles, provider, account, now = Date.now()) {
    const login = email(account?.email);
    const workspace = text(account?.account_id);
    if (provider !== "codex" || !login || !workspace) return null;
    return (Array.isArray(profiles) ? profiles : [])
      .map((profile) => seatAnnotationForProfile(profile, now))
      .filter((annotation) => annotation && annotation.email === login && annotation.workspace_id === workspace)
      .sort((a, b) => b.verified_at_ms - a.verified_at_ms)[0] || null;
  }

  function codexSource(fetch, payload) {
    const fetchedAt = Number.isFinite(fetch?.at) ? fetch.at : 0;
    const storedAt = Number.isFinite(payload?.codex_fetched_at_ms) ? payload.codex_fetched_at_ms : 0;
    if (fetch?.state === "done" && fetchedAt >= storedAt) {
      return { rows: distinctWorkspaces(fetch.rows), at: fetchedAt, stored: false };
    }
    if (payload?.codex_workspaces?.length) {
      return { rows: distinctWorkspaces(payload.codex_workspaces), at: storedAt, stored: true };
    }
    return { rows: [], at: 0, stored: false };
  }

  function accountScope(provider, type) {
    const plan = text(type).toLowerCase();
    if (provider === "claude") {
      const org = plan.replace(/^claude_/, "");
      if (["free", "pro", "max", "ultra"].includes(org)) return "personal";
      if (["team", "enterprise"].includes(org)) return "team";
    } else if (provider === "codex") {
      if (["free", "go", "plus", "prolite", "pro", "promax"].includes(plan)) return "personal";
      if (["team", "business", "enterprise"].includes(plan)) return "team";
    }
    return "";
  }

  // A group controls placement only. Even when two rows share an email,
  // each retains its own workspace, usage, freshness and connection state.
  function groupAccountRows(rows) {
    const groups = [];
    const byEmail = new Map();
    for (const row of rows) {
      const login = email(row.email);
      let group = byEmail.get(login);
      if (!group) {
        group = { email: login, rows: [] };
        groups.push(group);
        if (login) byEmail.set(login, group);
      }
      group.rows.push(row);
    }
    const priority = { personal: 0, team: 1 };
    for (const group of groups) {
      group.rows.sort((a, b) => (priority[a.scope] ?? 2) - (priority[b.scope] ?? 2));
    }
    return groups;
  }

  function organizationLabel(provider, account, workspaceName, locale = "ko", scope = accountScope(provider, account?.org_type)) {
    const org = text(workspaceName) || text(account?.org);
    const labels = String(locale).startsWith("en") ? { personal: "Personal", team: "Team" }
      : String(locale).startsWith("zh") ? { personal: "个人", team: "团队" } : { personal: "개인", team: "팀" };
    if (scope === "team") return org && org.toLowerCase() !== labels.team.toLowerCase() ? `${labels.team} · ${org}` : labels.team;
    if (scope === "personal") {
      return provider === "codex" && org && !["personal", labels.personal.toLowerCase()].includes(org.toLowerCase())
        ? `${labels.personal} · ${org}` : labels.personal;
    }
    return org;
  }

  return { email, workspaceKey, accountKey, distinctWorkspaces, activeWorkspace, claudeUsageForAccount, antigravityUsageForAccount, connectionsForAccount, seatAnnotationForProfile, seatAnnotationForAccount, codexSource, accountScope, groupAccountRows, organizationLabel };
});
