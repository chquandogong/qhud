// Public web list prices reviewed 2026-11-24. This catalog does not read invoices.
// https://help.openai.com/en/articles/9793128-about-chatgpt-pro-tiers
// https://help.openai.com/en/articles/11989085-what-is-chatgpt-go
// https://help.openai.com/en/articles/8792828-chatgpt-business-overview
// https://openai.com/index/introducing-chatgpt-go/
// https://claude.com/pricing
// https://support.claude.com/en/articles/9266767-what-is-the-team-plan
// https://one.google.com/about/google-ai-plans/?hl=en_US
// https://one.google.com/about/plans?hl=ko&gl=KR
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
  // Both services publish US rates per seat per month for each billing cycle.
  const TEAM_SEATS = { standard: { m: 25, y: 20 }, premium: { m: 125, y: 100 } };
  const TEXT = {
    ko: {
      unknown: "가격 미확인",
      missing: "이 워크스페이스의 월 정가를 확인할 정보가 없습니다.",
      missingSeat: "이 팀의 좌석 요금을 확인할 결제 정보가 없습니다.",
      unpricedSeat: "이 팀 좌석에 적용되는 정가를 확인할 수 없습니다.",
      missingAnnual: "이 요금제의 연 결제 정가를 확인할 수 없습니다.",
      amount: (usd) => `미국 웹 월 정가 ${usd} (USD)`,
      seat: (name, annual, monthly) => `${name} 미국 웹 좌석당 월 정가 (USD) · 연 결제 시 월 ${annual}, 월 결제 시 ${monthly}`,
      from: "배정 좌석 미확인 · 칩은 가장 낮은 공개 좌석 정가를 표시합니다.",
      annual: (year, effective, month) => `미국 웹 월 정가 (USD) · 연 ${year} 선결제 시 월 약 ${effective}, 월 결제 시 ${month}`,
      m: "m · 이 워크스페이스의 결제 주기는 월간입니다.",
      y: "y · 이 워크스페이스는 연간 결제하며, 칩은 월 환산 정가입니다.",
      unverified: "결제 주기 미확인 · 비교를 위해 월 결제 정가를 표시합니다.",
      observed: (date) => `결제 주기 확인일 · ${date}`,
      monthlyOnly: (source) => `m · ${source}에 따르면 이 요금제는 월 결제만 제공합니다.`,
      reference: `가격 막대 기준 $200 · 정가 확인 ${REVIEWED_AT}`,
    },
    en: {
      unknown: "Price unknown",
      missing: "The monthly list price is unknown for this workspace.",
      missingSeat: "The billing details for this team seat are unknown.",
      unpricedSeat: "The list price for this team seat is unavailable.",
      missingAnnual: "The annual list price for this plan is unavailable.",
      amount: (usd) => `US web list price ${usd}/month (USD)`,
      seat: (name, annual, monthly) => `${name} US list price per seat per month (USD) · ${annual} billed annually or ${monthly} billed monthly`,
      from: "Seat type unverified · the chip shows the lowest published seat price.",
      annual: (year, effective, month) => `US web list price (USD) · ${year} billed annually (about ${effective}/month), or ${month} billed monthly`,
      m: "m · This workspace is billed monthly.",
      y: "y · This workspace is billed annually. The chip shows the monthly equivalent.",
      unverified: "Billing cycle unverified · the chip shows the monthly list price for comparison.",
      observed: (date) => `Billing cycle verified on ${date}`,
      monthlyOnly: (source) => `m · ${source} lists monthly billing as the only option for this plan.`,
      reference: `Price bar reference $200 · list prices reviewed ${REVIEWED_AT}`,
    },
    zh: {
      unknown: "价格未确认",
      missing: "此工作区的每月定价尚未确认。",
      missingSeat: "该团队席位的计费信息尚未确认。",
      unpricedSeat: "此团队席位的定价尚未确认。",
      missingAnnual: "此套餐的年付定价尚未确认。",
      amount: (usd) => `美国网页版每月定价 ${usd}（美元）`,
      seat: (name, annual, monthly) => `${name} 美国官网每席位月价（美元）· 按年付费折合每月 ${annual}，按月付费 ${monthly}`,
      from: "分配的席位类型尚未确认，标签显示公开席位价格的起价。",
      annual: (year, effective, month) => `美国网页版定价（美元）· 按年预付 ${year}，折合每月约 ${effective}，按月付费 ${month}`,
      m: "m · 此工作区按月付费。",
      y: "y · 此工作区按年付费，标签显示折合月价。",
      unverified: "付款周期尚未确认，标签显示按月付费的公开价格以供比较。",
      observed: (date) => `付款周期确认日期 · ${date}`,
      monthlyOnly: (source) => `m · 根据 ${source}，此套餐仅提供按月付费。`,
      reference: `价格条基准 $200 · 定价核实日期 ${REVIEWED_AT}`,
    },
  };

  const norm = (value) => typeof value === "string" ? value.trim().toLowerCase() : "";
  const monthlyEquivalent = (year) => Math.round(year / 12 * 100) / 100;

  // The caller validates workspace ownership for seat annotations, observed
  // plans and billingCycle. A member's usage grade cannot establish billing.
  function priceFor(facts = {}, locale = "ko") {
    const language = norm(locale);
    const t = TEXT[language.startsWith("en") ? "en" : language.startsWith("zh") ? "zh" : "ko"];
    const cycle = facts.billingCycle === "annual" ? "y" : facts.billingCycle === "monthly" ? "m" : undefined;
    const unknown = (reason) => ({ kind: "unknown", min: null, max: null, label: t.unknown, detail: reason });

    function amount(usd, label, detail, mark = cycle, basis = t[cycle || "unverified"]) {
      const at = facts.billingVerifiedAt;
      const observed = usd > 0 && cycle && Number.isSafeInteger(at) && at > 0 && at <= Date.now()
        && t.observed(new Date(at).toLocaleDateString(t === TEXT.ko ? "ko-KR" : t === TEXT.zh ? "zh-CN" : "en-US"));
      const price = { kind: "amount", min: usd, max: usd, label,
        detail: [detail, usd > 0 && basis, observed, t.reference].filter(Boolean).join("\n") };
      if (usd > 0 && mark) price.cycle = mark;
      return price;
    }

    function monthlyOnly(usd, source) {
      if (usd > 0 && cycle === "y") return unknown(t.missingAnnual);
      return amount(usd, `$${usd}`, t.amount(`$${usd}`), "m", t.monthlyOnly(source));
    }

    function annualPlan(year, month) {
      const effective = monthlyEquivalent(year);
      const usd = cycle === "y" ? effective : month;
      return amount(usd, `$${usd}`, t.annual(`$${year}`, `$${effective}`, `$${month}`));
    }

    function teamPlan(name, seatType) {
      const seatKnown = Object.hasOwn(TEAM_SEATS, seatType);
      const rate = seatKnown ? TEAM_SEATS[seatType] : TEAM_SEATS.standard;
      const usd = rate[cycle || "m"];
      const rateDetail = (seatName, rate) => t.seat(seatName, `$${rate.y}`, `$${rate.m}`);
      if (seatKnown) return amount(usd, `$${usd}`, rateDetail(name, rate));
      return amount(usd, `$${usd}+`, [name, t.from, rateDetail("Standard", TEAM_SEATS.standard),
        rateDetail("Premium", TEAM_SEATS.premium)].join("\n"));
    }

    if (facts.provider === "codex") {
      const plan = norm(facts.planType);
      if (Object.hasOwn(CODEX_PERSONAL, plan)) return monthlyOnly(CODEX_PERSONAL[plan], "OpenAI Help Center");
      const seat = facts.seatAnnotation?.seat_type;
      if (plan === "team" || plan === "business" || (!plan && Object.hasOwn(TEAM_SEATS, seat))) {
        return teamPlan("OpenAI Help Center · ChatGPT Business", seat);
      }
      return unknown(plan === "enterprise" ? t.missingSeat : t.missing);
    }
    if (facts.provider === "claude") {
      const org = norm(facts.orgType);
      const organization = plans.claudeOrganizationPlanLabel(org);
      if (organization === "Team") {
        if (facts.seatType && !Object.hasOwn(TEAM_SEATS, facts.seatType)) return unknown(t.unpricedSeat);
        return teamPlan("Claude Help Center · Claude Team", facts.seatType);
      }
      if (organization === "Enterprise") return unknown(t.missingSeat);
      if (!org && facts.observedPlan?.plan === "free") return amount(0, "$0", t.amount("$0"));
      // The legacy claude_max organization type covers both Max and Ultra.
      switch (plans.claudePlanDisplay(null, org, facts.tiers)) {
        case "Free": return amount(0, "$0", t.amount("$0"));
        case "Pro": return annualPlan(200, 20);
        case "Max 5x": return monthlyOnly(100, "Claude Pricing");
        case "Ultra": return monthlyOnly(200, "Claude Pricing");
      }
    }
    if (facts.provider === "agy" && facts.planName === "Google AI Pro") {
      const usYear = 199.99, usMonth = 19.99;
      if (t === TEXT.ko) {
        const usd = cycle === "y" ? monthlyEquivalent(usYear) : usMonth;
        return amount(usd, cycle === "y" ? "₩24,167" : "₩29,000", [
          "Google AI Pro 대한민국 웹 정가 (KRW) · 월 결제 ₩29,000",
          "연 결제 ₩290,000 · 월 환산 약 ₩24,167",
          `막대 길이는 같은 결제 주기의 미국 월 환산 정가 $${usd} 기준`,
        ].join("\n"));
      }
      return annualPlan(usYear, usMonth);
    }
    return unknown(t.missing);
  }

  return { priceFor };
});
