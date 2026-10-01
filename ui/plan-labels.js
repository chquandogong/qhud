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

  return { codexPlanLabel, codexPlanDisplay };
});
