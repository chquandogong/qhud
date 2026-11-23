// Presentation names for provider plan values. Provider strings remain in the
// payload and diagnostics; only values reviewed here may reach a visible badge.
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root && root.document) root.QhudPlanLabels = api;
})(typeof window === "undefined" ? null : window, function () {
  "use strict";

  // D-022: exact status labels from the upstream Codex client. Keep this
  // allowlist narrow; unknown provider values belong in detail, not the badge.
  const CODEX_PLAN_LABELS = Object.freeze({
    prolite: "Pro 100",
    pro: "Pro 200",
    promax: "Pro 500",
  });
  function codexPlanLabel(planType) {
    if (typeof planType !== "string") return null;
    const key = planType.trim().toLowerCase();
    return CODEX_PLAN_LABELS[key] || null;
  }

  // accounts.json remains an explicit operator override. Otherwise only a
  // reviewed mapping is displayed; an unknown wire enum never leaks into UI.
  function codexPlanDisplay(override, planType) {
    if (typeof override === "string" && override.trim()) return override;
    return codexPlanLabel(planType);
  }

  // The native app supplies a display name from the queried user's tier.
  // A complete newer response without that name clears the old snapshot;
  // quota shapes and upgrade prompts are never subscription evidence.
  function antigravityPlanDisplay(override, snapshot, freshUsage = null) {
    if (typeof override === "string" && override.trim()) return override;
    const name = (freshUsage || snapshot)?.plan_name;
    if (typeof name !== "string" || /[\u0000-\u001f\u007f]/.test(name)) return null;
    return name.trim() || null;
  }

  function claudeOrganizationPlanLabel(orgType) {
    const type = typeof orgType === "string" ? orgType.trim().toLowerCase() : "";
    if (type === "claude_team" || type === "team") return "Team";
    if (type === "claude_enterprise" || type === "enterprise") return "Enterprise";
    return null;
  }

  const claudeUsageGrade = (tier) => tier === "default_claude_max_5x" ? "Max 5x"
    : tier === "default_claude_max_20x" ? "Ultra" : null;

  // The badge describes the known usage grade. For Team/Enterprise this is
  // the member's explicit user tier; the organization plan remains separate
  // in details. Unknown member tiers never inherit an organization's tier.
  function claudePlanDisplay(override, orgType, tiers = []) {
    if (typeof override === "string" && override.trim()) return override;
    if (typeof orgType !== "string") return null;
    const type = orgType.trim().toLowerCase();
    const knownTiers = Array.isArray(tiers) ? tiers : [];
    const organizationPlan = claudeOrganizationPlanLabel(type);
    if (organizationPlan) {
      const userTier = knownTiers.find((item) => item?.kind === "user");
      return claudeUsageGrade(userTier?.tier) || organizationPlan;
    }
    if (type === "claude_ultra" || type === "ultra") return "Ultra";
    if (type === "claude_pro" || type === "pro") return "Pro";
    if (type !== "claude_max" && type !== "max") return null;
    const tier = knownTiers.find((item) => item?.kind === "org")
      || knownTiers.find((item) => item?.kind === "user");
    return claudeUsageGrade(tier?.tier) || "Max";
  }

  return { codexPlanLabel, codexPlanDisplay, antigravityPlanDisplay, claudePlanDisplay, claudeOrganizationPlanLabel };
});
