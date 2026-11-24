// US web list prices reviewed 2026-11-24. This catalog does not read invoices.
// https://help.openai.com/en/articles/9793128-about-chatgpt-pro-tiers
// https://help.openai.com/en/articles/8792828-chatgpt-business-overview
// https://openai.com/index/introducing-chatgpt-go/
// https://claude.com/pricing
// https://support.claude.com/en/articles/9266767-what-is-the-team-plan
// https://one.google.com/about/google-ai-plans/?hl=en_US
(function (root, factory) {
  if (typeof module === "object" && module.exports) {
    module.exports = factory(require("./plan-labels.js"));
  } else if (root && root.document) {
    root.QhudPlanPrices = factory(root.QhudPlanLabels);
  }
})(typeof window === "undefined" ? null : window, function (plans) {
  "use strict";

  const REVIEWED_AT = "2026-11-24";
  const CODEX_PERSONAL = { free: 0, go: 8, plus: 20, prolite: 100, pro: 200, promax: 500 };
  // ChatGPT Business and Claude Team currently publish the same US seat rates.
  // Each pair contains the annual price per month, then the monthly price.
  const TEAM_SEATS = { standard: [20, 25], premium: [100, 125] };
  const TEXT = {
    ko: {
      unknown: "가격 미확인",
      missing: "이 워크스페이스의 월 정가를 확인할 정보가 없습니다.",
      missingSeat: "이 팀의 좌석 요금을 확인할 결제 정보가 없습니다.",
      amount: (usd) => `미국 웹 월 정가 ${usd} (USD)`,
      seat: (annual, monthly) => `미국 웹 좌석당 월 정가 (USD) · 연 결제 시 월 ${annual}, 월 결제 시 ${monthly}`,
      teamRates: (plan) => `${plan} 미국 좌석당 월 정가 (USD) · 연 결제 월 환산 / 월 결제\n배정된 좌석 유형과 결제 주기는 미확인입니다.`,
      annual: (year, effective, month) => `미국 웹 월 정가 (USD) · 연 ${year} 선결제 시 월 약 ${effective}, 월 결제 시 ${month}`,
      reference: `가격 막대 기준 $200 · 정가 확인 ${REVIEWED_AT}`,
    },
    en: {
      unknown: "Price unknown",
      missing: "The monthly list price is unknown for this workspace.",
      missingSeat: "The billing details for this team seat are unknown.",
      amount: (usd) => `US web list price ${usd}/month (USD)`,
      seat: (annual, monthly) => `US web list price per seat per month (USD) · ${annual} billed annually or ${monthly} billed monthly`,
      teamRates: (plan) => `${plan} US list prices per seat per month (USD) · annual billing / monthly billing\nThe assigned seat and billing cycle are unverified.`,
      annual: (year, effective, month) => `US web list price (USD) · ${year} billed annually (about ${effective}/month), or ${month} billed monthly`,
      reference: `Price bar reference $200 · list prices reviewed ${REVIEWED_AT}`,
    },
    zh: {
      unknown: "价格未确认",
      missing: "此工作区的每月定价尚未确认。",
      missingSeat: "该团队席位的计费信息尚未确认。",
      amount: (usd) => `美国网页版每月定价 ${usd}（美元）`,
      seat: (annual, monthly) => `美国网页版每席位每月定价（美元）· 按年付费折合每月 ${annual}，按月付费 ${monthly}`,
      teamRates: (plan) => `${plan} 美国官网每席位月价（美元）· 年付折合月价 / 月付价格\n当前席位类型和付款周期尚未确认。`,
      annual: (year, effective, month) => `美国网页版定价（美元）· 按年预付 ${year}，折合每月约 ${effective}，按月付费 ${month}`,
      reference: `价格条基准 $200 · 定价核实日期 ${REVIEWED_AT}`,
    },
  };

  const norm = (value) => typeof value === "string" ? value.trim().toLowerCase() : "";

  // Only pass a seat annotation after matching its email and workspace with
  // account-identities, or an observed plan accepted by configuredConnectionView.
  // Display aliases and organization/member usage grades cannot establish seat prices.
  function priceFor(facts = {}, locale = "ko") {
    const language = norm(locale);
    const t = TEXT[language.startsWith("en") ? "en" : language.startsWith("zh") ? "zh" : "ko"];
    const unknown = (reason) => ({ kind: "unknown", min: null, max: null, label: t.unknown, detail: reason });
    const amount = (usd) => ({
      kind: "amount", min: usd, max: usd, label: `$${usd}`,
      detail: `${t.amount(`$${usd}`)}\n${t.reference}`,
    });
    const range = (min, max, detail, label = `$${min}–${max}`) => ({
      kind: "range", min, max, label, detail: `${detail}\n${t.reference}`,
    });
    const annualPlan = (year, month) => {
      const effective = Math.round(year / 12 * 100) / 100;
      return range(effective, month, t.annual(`$${year}`, `$${effective}`, `$${month}`),
        `≈$${Math.round(effective)}–${Math.round(month)}`);
    };
    const teamPlan = (name) => {
      const [standardAnnual, standardMonthly] = TEAM_SEATS.standard;
      const [premiumAnnual, premiumMonthly] = TEAM_SEATS.premium;
      return range(standardAnnual, premiumMonthly, [
        t.teamRates(name),
        `Standard · $${standardAnnual} / $${standardMonthly}`,
        `Premium · $${premiumAnnual} / $${premiumMonthly}`,
      ].join("\n"));
    };

    if (facts.provider === "codex") {
      const plan = norm(facts.planType);
      if (Object.hasOwn(CODEX_PERSONAL, plan)) return amount(CODEX_PERSONAL[plan]);
      const seat = facts.seatAnnotation?.seat_type;
      const isBusiness = plan === "team" || plan === "business";
      if ((!plan || isBusiness) && Object.hasOwn(TEAM_SEATS, seat)) {
        const [annual, monthly] = TEAM_SEATS[seat];
        return range(annual, monthly, t.seat(`$${annual}`, `$${monthly}`));
      }
      if (isBusiness) return teamPlan("OpenAI Help Center · ChatGPT Business");
      return unknown(plan === "enterprise" ? t.missingSeat : t.missing);
    }
    if (facts.provider === "claude") {
      const org = norm(facts.orgType);
      const organization = plans.claudeOrganizationPlanLabel(org);
      if (organization === "Team") return teamPlan("Claude Help Center · Claude Team");
      if (organization === "Enterprise") return unknown(t.missingSeat);
      if (!org && facts.observedPlan?.plan === "free") return amount(0);
      // The legacy claude_max organization type covers both Max and Ultra.
      // Reuse the reviewed tier mapping, without the user's display override.
      switch (plans.claudePlanDisplay(null, org, facts.tiers)) {
        case "Free": return amount(0);
        case "Pro": return annualPlan(200, 20);
        case "Max 5x": return amount(100);
        case "Ultra": return amount(200);
      }
    }
    if (facts.provider === "agy" && facts.planName === "Google AI Pro") return annualPlan(199.99, 19.99);
    return unknown(t.missing);
  }

  return { priceFor };
});
