const test = require("node:test");
const assert = require("node:assert/strict");
const { priceFor } = require("../ui/plan-prices.js");

function expectPrice(facts, kind, min, max, label) {
  const price = priceFor(facts, "en-US");
  assert.deepEqual([price.kind, price.min, price.max, price.label], [kind, min, max, label]);
  return price;
}

test("Codex monthly prices use the workspace plan and keep the $500 to $200 ratio", () => {
  for (const [planType, usd] of Object.entries({ free: 0, go: 8, plus: 20, prolite: 100, pro: 200, promax: 500 })) {
    const price = expectPrice({ provider: "codex", planType, override: "Custom" }, "amount", usd, usd, `$${usd}`);
    assert.match(price.detail, /USD/);
    assert.match(price.detail, /\$200/);
  }
  const codex = (planType) => priceFor({ provider: "codex", planType });
  assert.equal(codex("promax").min / codex("pro").min, 2.5);
  for (const planType of ["Pro 200", "Max 5x", "constructor", "__proto__", null]) {
    expectPrice({ provider: "codex", planType, override: "Pro 500" }, "unknown", null, null, "Price unknown");
  }
});

test("a previously verified Business seat displays its US annual/monthly range", () => {
  const standard = { seat_type: "standard" };
  const premium = { seat_type: "premium" };
  for (const planType of ["team", "business", undefined]) {
    expectPrice({ provider: "codex", planType, seatAnnotation: standard }, "range", 20, 25, "$20–25");
    expectPrice({ provider: "codex", planType, seatAnnotation: premium }, "range", 100, 125, "$100–125");
  }
  expectPrice({ provider: "codex", planType: "enterprise", seatAnnotation: premium }, "unknown", null, null, "Price unknown");
  expectPrice({ provider: "codex", planType: "free", seatAnnotation: standard }, "amount", 0, 0, "$0");
});

test("known team plans without a confirmed seat show the published Standard and Premium prices", () => {
  const teamMember = {
    provider: "claude", orgType: "claude_team",
    tiers: [{ kind: "user", tier: "default_claude_max_5x" }],
    seatAnnotation: { seat_type: "premium" }, observedPlan: { plan: "free" },
  };
  for (const [facts, planName] of [
    [{ provider: "codex", planType: "team" }, "ChatGPT Business"],
    [{ provider: "codex", planType: "business", seatAnnotation: { seat_type: "unknown" } }, "ChatGPT Business"],
    [{ provider: "claude", orgType: "team" }, "Claude Team"],
    [teamMember, "Claude Team"],
  ]) {
    const price = expectPrice(facts, "range", 20, 125, "$20–125");
    for (const text of [planName, "Help Center", "2026-11-24", "Standard · $20 / $25", "Premium · $100 / $125",
      "annual billing / monthly billing", "unverified"]) {
      assert.ok(price.detail.includes(text));
    }
  }
});

test("Claude individual prices follow organization and reviewed tier evidence", () => {
  const check = (orgType, tier, kind, min, max, label) => expectPrice({
    provider: "claude", orgType, tiers: [{ kind: "org", tier }],
  }, kind, min, max, label);
  check("claude_max", "default_claude_max_5x", "amount", 100, 100, "$100");
  check("claude_max", "default_claude_max_20x", "amount", 200, 200, "$200");
  check("claude_ultra", null, "amount", 200, 200, "$200");
  check("claude_max", "unrecognized", "unknown", null, null, "Price unknown");
  check("claude_pro", null, "range", 16.67, 20, "≈$17–20");
  check("claude_free", null, "amount", 0, 0, "$0");
  expectPrice({ provider: "claude", observedPlan: { plan: "free" } }, "amount", 0, 0, "$0");
  expectPrice({ provider: "claude", orgType: "claude_ultra", observedPlan: { plan: "free" } }, "amount", 200, 200, "$200");
});

test("a team scope alone or a negotiated Enterprise plan cannot inherit the public team rates", () => {
  for (const facts of [
    { provider: "codex", scope: "team" }, { provider: "claude", scope: "team" },
    { provider: "codex", planType: "enterprise" },
    { provider: "claude", orgType: "claude_enterprise", tiers: [{ kind: "user", tier: "default_claude_max_5x" }],
      seatAnnotation: { seat_type: "premium" }, observedPlan: { plan: "free" } },
  ]) {
    expectPrice(facts, "unknown", null, null, "Price unknown");
  }
});

test("Google AI Pro uses only the exact native name and preserves unrounded dollar amounts in the detail", () => {
  const price = expectPrice({ provider: "agy", planName: "Google AI Pro" }, "range", 16.67, 19.99, "≈$17–20");
  assert.match(price.detail, /\$199\.99 billed annually/);
  assert.match(price.detail, /\$19\.99 billed monthly/);
  for (const facts of [
    { provider: "agy", override: "Google AI Pro" },
    { provider: "agy", planName: "Google AI Pro Trial" },
    { provider: "claude", planName: "Google AI Pro" },
    { provider: "agy", observedPlan: { plan: "free" } },
  ]) {
    expectPrice(facts, "unknown", null, null, "Price unknown");
  }
});

test("chip labels and dated monthly price details are available for each UI locale", () => {
  for (const [locale, unknown, month] of [["ko-KR", "가격 미확인", "월"], ["zh-CN", "价格未确认", "月"], ["en-US", "Price unknown", "month"]]) {
    assert.equal(priceFor({}, locale).label, unknown);
    const price = priceFor({ provider: "codex", planType: "team", seatAnnotation: { seat_type: "standard" } }, locale);
    for (const text of [month, "$20", "$25", "2026-11-24"]) assert.ok(price.detail.includes(text));
  }
});
