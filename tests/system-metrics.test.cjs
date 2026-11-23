const test = require("node:test");
const assert = require("node:assert/strict");
const metrics = require("../ui/system-metrics.js");
const plans = require("../ui/plan-labels.js");

test("unknown readings stay unknown; measured zero is a valid idle reading", () => {
  for (const value of [null, undefined, NaN, Infinity, -1, "0"]) {
    assert.equal(metrics.formatRate(value), "--");
    assert.equal(metrics.formatPercent(value), "--");
  }
  assert.equal(metrics.formatRate(0), "0 B/s");
  assert.equal(metrics.formatPercent(0), "0%");
  assert.equal(metrics.formatPercent(240), "100%");
  assert.equal(metrics.combinedRate({ rx: 0, tx: 0 }, "rx", "tx"), 0);
  assert.equal(metrics.combinedRate({ rx: 100, tx: null }, "rx", "tx"), null);
});

test("throughput uses byte-per-second units, including rounded unit boundaries", () => {
  assert.equal(metrics.formatRate(2_400_000), "2.4 MB/s");
  assert.equal(metrics.formatRate(2_400_000, true), "2.4M/s");
  assert.equal(metrics.formatRate(70_000, true), "70K/s");
  assert.equal(metrics.formatRate(999_999), "1 MB/s");
  assert.equal(metrics.formatBytes(32_000_000_000), "32 GB");
});

test("time slots preserve outages and replace duplicate reports without inventing history", () => {
  const snapshot = {
    sampled_at_ms: 100_000, interval_ms: 2000,
    history: [
      { at_ms: 96_000, cpu_pct: 11 },
      { at_ms: 100_000, cpu_pct: 20 },
      { at_ms: 90_000, cpu_pct: 0 },
      { at_ms: 40_000, cpu_pct: 100 }, // older than the displayed minute
      { at_ms: 102_000, cpu_pct: 100 }, // future samples must not wrap
      { at_ms: null, cpu_pct: 100 },
    ],
    current: { at_ms: 100_000, cpu_pct: 24 },
  };
  const slots = metrics.historySlots(snapshot);
  assert.equal(slots.length, 30);
  assert.equal(slots.at(-1).cpu_pct, 24);
  assert.equal(slots.at(-2), null); // missing 98 s is a gap
  assert.equal(slots.at(-3).cpu_pct, 11);
  assert.equal(slots.at(-6).cpu_pct, 0);
  assert.equal(slots.filter(Boolean).length, 3);
  assert.deepEqual(metrics.historySlots(snapshot), slots);
  assert.equal(metrics.historySlots(undefined).filter(Boolean).length, 0);
});

test("small sampling jitter stays in its time slot while a long pause leaves gaps", () => {
  const slots = metrics.historySlots({
    sampled_at_ms: 60_100, interval_ms: 2000,
    history: [{ at_ms: 58_040, cpu_pct: 12 }, { at_ms: 50_080, cpu_pct: 33 }],
    current: { at_ms: 60_100, cpu_pct: null },
  });
  assert.equal(slots.at(-1).cpu_pct, null);
  assert.equal(slots.at(-2).cpu_pct, 12);
  assert.equal(slots.at(-3), null);
  assert.equal(slots.at(-6).cpu_pct, 33);
});

test("throughput scale accommodates bursts, retains peaks, and releases slowly", () => {
  const floor = 1_000_000;
  assert.equal(metrics.nextScale(undefined, [0, null], floor), floor);
  const burst = metrics.nextScale(floor, [8_000_000], floor);
  assert.ok(burst >= 8_000_000);
  assert.equal(metrics.nextScale(burst, [8_000_000, 0], floor), burst);
  assert.equal(metrics.nextScale(burst, [0], floor, 0), burst);
  const falling = metrics.nextScale(burst, [0], floor, 2000);
  assert.ok(falling < burst && falling > burst * 0.9);
  assert.equal(metrics.nextScale(burst, [0], floor, 600_000), floor);
});

test("browser fixtures match the real optional-GPU report contract", () => {
  const supported = metrics.demoSnapshot(100_000, true);
  const unsupported = metrics.demoSnapshot(100_000, false);
  assert.equal(supported.history.length, 30);
  assert.equal(supported.current.at_ms, supported.sampled_at_ms);
  assert.equal(supported.history[0].at_ms, 42_000);
  assert.equal(supported.gpu.name, "Demo GPU");
  assert.equal(unsupported.gpu, null);
  assert.ok(unsupported.history.every((sample) => sample.gpu_pct === null));
});

test("Codex Pro wire values use the current product names", () => {
  assert.equal(plans.codexPlanLabel("prolite"), "Pro 100");
  assert.equal(plans.codexPlanLabel("pro"), "Pro 200");
  assert.equal(plans.codexPlanLabel("promax"), "Pro 500");
  assert.equal(plans.codexPlanLabel(" PRO "), "Pro 200");
});

test("Codex plan labels keep explicit overrides and hide unknown wire values", () => {
  assert.equal(plans.codexPlanDisplay(" Research seat ", "pro"), " Research seat ");
  assert.equal(plans.codexPlanDisplay("", "prolite"), "Pro 100");
  assert.equal(plans.codexPlanDisplay(null, "Pro-Max"), null);
  assert.equal(plans.codexPlanDisplay(null, "future_plan"), null);
  assert.equal(plans.codexPlanDisplay(null, null), null);
});

test("Antigravity displays the account's supplied plan name without inferring it from usage or upgrade prompts", () => {
  assert.equal(plans.antigravityPlanDisplay(null, { plan_name: "Google AI Pro" }), "Google AI Pro");
  assert.equal(plans.antigravityPlanDisplay(null, { plan_name: " Google AI Ultra " }), "Google AI Ultra");
  assert.equal(plans.antigravityPlanDisplay(null, { five_hour: { pct: 20 }, upgrade_prompt: "Google AI Pro" }), null);
  for (const plan_name of [null, undefined, "", "   ", "Google\nAI Pro", 12]) {
    assert.equal(plans.antigravityPlanDisplay(null, { plan_name }), null);
  }
});

test("Antigravity fresh plan data replaces or clears only its matched snapshot and preserves explicit overrides", () => {
  const saved = { account_id: "one@example.com", plan_name: "Google AI Pro" };
  assert.equal(plans.antigravityPlanDisplay(null, saved, { plan_name: "Google AI Ultra" }), "Google AI Ultra");
  assert.equal(plans.antigravityPlanDisplay(null, saved, { plan_name: null }), null);
  assert.equal(plans.antigravityPlanDisplay(null, saved, {}), null);
  assert.equal(plans.antigravityPlanDisplay(" Custom plan ", saved, {}), " Custom plan ");
  assert.equal(plans.antigravityPlanDisplay(null, { account_id: "other@example.com" }), null);
});

test("Claude personal usage grades show Ultra or Max 5x from known metadata", () => {
  assert.equal(plans.claudePlanDisplay(null, "claude_free"), "Free");
  assert.equal(plans.claudePlanDisplay(null, "claude_max", [{ kind: "org", tier: "default_claude_max_20x" }]), "Ultra");
  assert.equal(plans.claudePlanDisplay(null, "claude_max", [{ kind: "org", tier: "default_claude_max_5x" }]), "Max 5x");
  assert.equal(plans.claudePlanDisplay(null, "claude_max", [{ kind: "user", tier: "default_claude_max_20x" }]), "Ultra");
  assert.equal(plans.claudePlanDisplay(null, "claude_ultra"), "Ultra");
  assert.equal(plans.claudePlanDisplay(null, " ultra "), "Ultra");
});

test("Claude usage grade preserves personal tier precedence and explicit overrides", () => {
  const personalUltra = [{ kind: "user", tier: "default_claude_max_20x" }];
  assert.equal(plans.claudePlanDisplay(null, "claude_team", personalUltra), "Ultra");
  assert.equal(plans.claudePlanDisplay(null, "claude_max", [{ kind: "org", tier: "default_claude_max_5x" }, ...personalUltra]), "Max 5x");
  assert.equal(plans.claudePlanDisplay(" Research seat ", "claude_max", personalUltra), " Research seat ");
  assert.equal(plans.claudePlanDisplay("", "claude_max", personalUltra), "Ultra");
  assert.equal(plans.claudePlanDisplay(null, "future_org", personalUltra), null);
  assert.equal(plans.claudePlanDisplay(null, null, personalUltra), null);
});

test("Team and Enterprise badges use explicit member grade while preserving the organization plan", () => {
  const tiers = [{ kind: "org", tier: "default_raven" }, { kind: "user", tier: "default_claude_max_5x" }];
  assert.equal(plans.claudePlanDisplay(null, "claude_team", tiers), "Max 5x");
  assert.equal(plans.claudePlanDisplay(null, "claude_enterprise", tiers), "Max 5x");
  assert.equal(plans.claudeOrganizationPlanLabel("claude_team"), "Team");
  assert.equal(plans.claudeOrganizationPlanLabel("claude_enterprise"), "Enterprise");
  assert.equal(plans.claudeOrganizationPlanLabel("claude_max"), null);
  assert.equal(plans.claudeOrganizationPlanLabel("future_org"), null);
  assert.equal(plans.claudePlanDisplay(null, "claude_team", [{ kind: "org", tier: "default_claude_max_20x" }, tiers[1]]), "Max 5x");
  assert.equal(plans.claudePlanDisplay("Custom seat", "claude_team", tiers), "Custom seat");
});

test("unknown Team member grades are not inferred from organization tiers", () => {
  const tiers = [{ kind: "org", tier: "default_claude_max_20x" }, { kind: "user", tier: "future_tier" }];
  assert.equal(plans.claudePlanDisplay(null, "claude_team", tiers), "Team");
  assert.equal(plans.claudePlanDisplay(null, "claude_enterprise", tiers), "Enterprise");
  assert.equal(plans.claudePlanDisplay(null, "claude_team", [tiers[0]]), "Team");
  assert.equal(plans.claudePlanDisplay(null, "claude_team", [{ kind: "user", tier: "toString" }]), "Team");
});
