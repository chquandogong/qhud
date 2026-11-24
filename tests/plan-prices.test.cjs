const test = require("node:test");
const assert = require("node:assert/strict");
const { priceFor } = require("../ui/plan-prices.js");

function expectAmount(facts, usd, label = `$${usd}`, cycle, locale = "en-US") {
  const price = priceFor(facts, locale);
  assert.deepEqual([price.kind, price.min, price.max, price.label, price.cycle], ["amount", usd, usd, label, cycle]);
  return price;
}

function expectUnknown(facts) {
  const price = priceFor(facts, "en-US");
  assert.deepEqual([price.kind, price.min, price.max, price.label, price.cycle],
    ["unknown", null, null, "Price unknown", undefined]);
  return price;
}

test("recognized individual Codex prices retain the $500 to $200 ratio and mark monthly-only plans", () => {
  for (const [planType, usd] of Object.entries({ go: 8, plus: 20, prolite: 100, pro: 200, promax: 500 })) {
    const price = expectAmount({ provider: "codex", planType, override: "Custom" }, usd, `$${usd}`, "m");
    for (const text of ["USD", "$200", "OpenAI Help Center", "monthly billing as the only option"]) assert.ok(price.detail.includes(text));
    expectUnknown({ provider: "codex", planType, billingCycle: "annual" });
  }
  const codex = (planType) => priceFor({ provider: "codex", planType });
  assert.equal(codex("promax").min / codex("pro").min, 2.5);
  for (const planType of ["Pro 200", "Max 5x", "constructor", "__proto__", null]) {
    expectUnknown({ provider: "codex", planType, override: "Pro 500" });
  }
});

test("Free prices never carry a billing cycle, including previously observed Claude Free plans", () => {
  for (const facts of [
    { provider: "codex", planType: "free", seatAnnotation: { seat_type: "standard" } },
    { provider: "claude", orgType: "claude_free", seatType: "premium" },
    { provider: "claude", observedPlan: { plan: "free" } },
  ]) {
    for (const billingCycle of [undefined, "monthly", "annual"]) {
      const price = expectAmount({ ...facts, billingCycle }, 0);
      assert.doesNotMatch(price.detail, /only option|unverified|billed annually|billed monthly/);
    }
  }
});

test("verified Codex and native Claude seat prices select the exact workspace billing cycle", () => {
  for (const [type, monthly, annual] of [["standard", 25, 20], ["premium", 125, 100]]) {
    for (const facts of [
      ...["team", "business", undefined].map(planType => ({ provider: "codex", planType, seatAnnotation: { seat_type: type } })),
      { provider: "claude", orgType: "claude_team", seatType: type, tiers: [{ kind: "user", tier: "default_claude_max_5x" }] },
    ]) {
      for (const billingCycle of [undefined, "unknown", "month", "y", "ANNUAL", {}]) {
        const price = expectAmount({ ...facts, billingCycle }, monthly);
        assert.match(price.detail, /Billing cycle unverified/);
        assert.match(price.detail, /monthly list price for comparison/);
      }
      const m = expectAmount({ ...facts, billingCycle: "monthly" }, monthly, `$${monthly}`, "m");
      assert.match(m.detail, /workspace is billed monthly/);
      const y = expectAmount({ ...facts, billingCycle: "annual" }, annual, `$${annual}`, "y");
      assert.match(y.detail, /workspace is billed annually/);
      assert.match(y.detail, /monthly equivalent/);
    }
  }
});

test("unverified team seats show one starting price and explain the two seat options", () => {
  for (const facts of [
    { provider: "codex", planType: "team" },
    { provider: "codex", planType: "business", seatAnnotation: { seat_type: "unknown" } },
    { provider: "claude", orgType: "team" },
    { provider: "claude", orgType: "claude_team", tiers: [{ kind: "user", tier: "default_claude_max_5x" }],
      seatAnnotation: { seat_type: "premium" }, observedPlan: { plan: "free" } },
  ]) {
    const price = expectAmount(facts, 25, "$25+");
    for (const text of ["Seat type unverified", "Standard", "$25", "$20", "Premium", "$125", "$100"]) {
      assert.ok(price.detail.includes(text));
    }
    expectAmount({ ...facts, billingCycle: "monthly" }, 25, "$25+", "m");
    expectAmount({ ...facts, billingCycle: "annual" }, 20, "$20+", "y");
  }
});

test("a reported Claude seat outside the catalog never inherits the standard starting price", () => {
  for (const billingCycle of [undefined, "monthly", "annual"]) {
    const price = expectUnknown({ provider: "claude", orgType: "claude_team", seatType: "other", billingCycle });
    assert.match(price.detail, /price for this team seat is unavailable/);
  }
});

test("Claude individual prices use reviewed plan evidence and monthly-equivalent annual rates", () => {
  for (const [orgType, tier, usd] of [
    ["claude_max", "default_claude_max_5x", 100], ["claude_max", "default_claude_max_20x", 200],
    ["claude_ultra", null, 200],
  ]) {
    const facts = { provider: "claude", orgType, tiers: [{ kind: "org", tier }], observedPlan: { plan: "free" } };
    assert.match(expectAmount(facts, usd, `$${usd}`, "m").detail, /Claude Pricing lists monthly billing/);
    expectUnknown({ ...facts, billingCycle: "annual" });
  }
  expectUnknown({ provider: "claude", orgType: "claude_max", tiers: [{ kind: "org", tier: "unrecognized" }] });
  const pro = { provider: "claude", orgType: "claude_pro" };
  expectAmount(pro, 20);
  expectAmount({ ...pro, billingCycle: "monthly" }, 20, "$20", "m");
  assert.match(expectAmount({ ...pro, billingCycle: "annual" }, 16.67, "$16.67", "y").detail, /\$200 billed annually/);
});

test("an unsupported plan or the team scope alone does not imply paid team rates", () => {
  for (const facts of [
    { provider: "codex", scope: "team" }, { provider: "claude", scope: "team" },
    { provider: "codex", planType: "enterprise", seatAnnotation: { seat_type: "premium" } },
    { provider: "claude", orgType: "claude_enterprise", tiers: [{ kind: "user", tier: "default_claude_max_5x" }],
      seatType: "premium", observedPlan: { plan: "free" } },
    { provider: "agy", override: "Google AI Pro" }, { provider: "agy", planName: "Google AI Pro Trial" },
    { provider: "claude", planName: "Google AI Pro" }, { provider: "agy", observedPlan: { plan: "free" } },
  ]) {
    expectUnknown({ ...facts, billingCycle: "monthly" });
  }
});

test("Google AI Pro selects the local monthly equivalent and scales in dollars for that cycle", () => {
  const facts = { provider: "agy", planName: "Google AI Pro" };
  for (const locale of ["ko", "ko-KR"]) {
    const fallback = expectAmount(facts, 19.99, "₩29,000", undefined, locale);
    for (const text of ["결제 주기 미확인", "월 결제 ₩29,000", "연 결제 ₩290,000", "$19.99"]) assert.ok(fallback.detail.includes(text));
    expectAmount({ ...facts, billingCycle: "monthly" }, 19.99, "₩29,000", "m", locale);
    const annual = expectAmount({ ...facts, billingCycle: "annual" }, 16.67, "₩24,167", "y", locale);
    for (const text of ["연 결제 ₩290,000", "₩24,167", "$16.67", "월 환산"]) assert.ok(annual.detail.includes(text));
  }
  assert.equal(priceFor(facts).label, "₩29,000");
  for (const locale of ["en-US", "zh-CN"]) {
    expectAmount(facts, 19.99, "$19.99", undefined, locale);
    const annual = expectAmount({ ...facts, billingCycle: "annual" }, 16.67, "$16.67", "y", locale);
    assert.ok(annual.detail.includes("$199.99"));
  }
});

test("comparison-price details are localized and dated", () => {
  for (const [locale, unknown, unverified] of [
    ["ko-KR", "가격 미확인", "결제 주기 미확인"], ["zh-CN", "价格未确认", "付款周期尚未确认"],
    ["en-US", "Price unknown", "Billing cycle unverified"],
  ]) {
    assert.equal(priceFor({}, locale).label, unknown);
    const price = expectAmount({ provider: "codex", planType: "team", seatAnnotation: { seat_type: "standard" } }, 25, "$25", undefined, locale);
    for (const text of [unverified, "$20", "$25", "2026-11-24"]) assert.ok(price.detail.includes(text));
  }
});
