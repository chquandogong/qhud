// Reviewed product guidance, not an account model catalog or a quota source.
// No requests: model access still depends on the provider's live model picker.
// Reviewed 2026-11-23:
// https://antigravity.google/docs/models
// https://antigravity.google/pricing
// https://support.claude.com/en/articles/11940350-claude-code-model-configuration
// https://support.claude.com/en/articles/15424964-claude-fable-models-on-your-plan
// https://learn.chatgpt.com/docs/pricing
// https://help.openai.com/en/articles/12003714-chatgpt-business-models-and-limits
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root && root.document) root.QhudModelGuide = api;
})(typeof window === "undefined" ? null : window, function () {
  "use strict";

  const REVIEWED_AT = "2026-11-23";
  const normalized = (value) => typeof value === "string" && value.length <= 160 &&
    !/[\x00-\x1f\x7f]/.test(value) ? value.trim().toLowerCase() : "";
  const localeKey = (locale) => String(locale || "ko").startsWith("en") ? "en"
    : String(locale || "ko").startsWith("zh") ? "zh" : "ko";

  const TEXT = {
    ko: {
      title: "지원 모델",
      source: (provider) => `${provider} · 요금제 안내 · ${REVIEWED_AT}`,
      codexBase: "지원 모델 예시: GPT-6.2 Astra · GPT-6.2 Sol · GPT-6.1 Terra",
      codexVersion: "Codex CLI 필요 버전: GPT-6.2는 0.166.0 이상, GPT-6.1 Terra는 0.159.0 이상.",
      codexBusiness: "Business 모델 접근은 좌석과 워크스페이스 권한에 따릅니다. Standard/Premium 모두 이 조건이 적용됩니다.",
      codexPicker: "이 계정의 모델은 Codex 앱 모델 선택기 또는 CLI /model에서 확인하세요.",
      claudeBase: "Claude Code: Opus 5.5 · Sonnet 5.5 · Haiku 5.5",
      fableIncluded: "Fable 5.2: 플랜에 포함, 공유 주간 한도의 최대 50% 사용. 초과 사용은 사용량 크레딧 필요.",
      fableCredits: "Fable 5.2: 기본 사용 한도에 미포함. 사용량 크레딧으로 별도 과금.",
      fableTeamUnknown: "Fable 5.2: Team Premium 좌석은 포함(공유 주간 한도의 최대 50%), Standard 좌석은 사용량 크레딧 과금. 좌석 유형 확인 필요.",
      claudeOrg: "모델 이용과 추가 과금은 조직 관리자의 설정을 따릅니다.",
      claudePicker: "이 계정의 모델은 Claude Code /model에서 확인하세요. Fable 5.2는 2.1.321 이상 필요.",
      agyBase: "Antigravity: Gemini 4 Neon · Argon · Hydrogen, Gemini 3.8 Flash, Claude Sonnet 5.5 · Opus 5.5 (Thinking)",
      agyConditions: "Neon·Argon·Orchestrator: 유료 Pro 또는 대상 프리미엄 기기 체험판. Hydrogen·Claude 5.5: 유료 Pro만 제공.",
      agyLegacy: "GPT-OSS-120b와 체험판의 Claude 4.6은 2026-11-30 제거 예정.",
      agyPicker: "이 계정의 모델은 Antigravity 앱 모델 선택기 또는 agy models에서 확인하세요.",
    },
    en: {
      title: "Supported models",
      source: (provider) => `${provider} · plan guide · ${REVIEWED_AT}`,
      codexBase: "Model examples: GPT-6.2 Astra · GPT-6.2 Sol · GPT-6.1 Terra",
      codexVersion: "Codex CLI required: 0.166.0+ for GPT-6.2; 0.159.0+ for GPT-6.1 Terra.",
      codexBusiness: "Business model access depends on seat and workspace permissions, for both Standard and Premium.",
      codexPicker: "Check this account's models in the Codex app picker or CLI /model.",
      claudeBase: "Claude Code: Opus 5.5 · Sonnet 5.5 · Haiku 5.5",
      fableIncluded: "Fable 5.2: included, using up to 50% of the shared weekly limit. Further usage requires usage credits.",
      fableCredits: "Fable 5.2: outside included usage limits; billed separately through usage credits.",
      fableTeamUnknown: "Fable 5.2: included on Team Premium seats (up to 50% of the shared weekly limit); usage credits on Standard seats. Seat type needs verification.",
      claudeOrg: "Model access and additional billing follow the organization's settings.",
      claudePicker: "Check this account's models in Claude Code /model. Fable 5.2 requires version 2.1.321+.",
      agyBase: "Antigravity: Gemini 4 Neon · Argon · Hydrogen, Gemini 3.8 Flash, Claude Sonnet 5.5 · Opus 5.5 (Thinking)",
      agyConditions: "Neon, Argon, Orchestrator: paid Pro or eligible premium-device trials. Hydrogen and Claude 5.5: paid Pro only.",
      agyLegacy: "GPT-OSS-120b and trial access to Claude 4.6 are scheduled for removal on 2026-11-30.",
      agyPicker: "Check this account's models in the Antigravity app picker or agy models.",
    },
    zh: {
      title: "支持模型",
      source: (provider) => `${provider} · 套餐指南 · ${REVIEWED_AT}`,
      codexBase: "模型示例：GPT-6.2 Astra · GPT-6.2 Sol · GPT-6.1 Terra",
      codexVersion: "Codex CLI 最低版本：GPT-6.2 为 0.166.0；GPT-6.1 Terra 为 0.159.0。",
      codexBusiness: "Business 模型访问取决于席位与工作区权限，Standard 和 Premium 均适用。",
      codexPicker: "在 Codex 应用模型选择器或 CLI /model 中查看当前账户的模型。",
      claudeBase: "Claude Code：Opus 5.5 · Sonnet 5.5 · Haiku 5.5",
      fableIncluded: "Fable 5.2：包含在套餐内，最多使用共享周限额的 50%。超过后需要用量点数。",
      fableCredits: "Fable 5.2：不包含在套餐用量限额内，通过用量点数单独计费。",
      fableTeamUnknown: "Fable 5.2：Team Premium 席位包含（最多使用共享周限额的 50%）；Standard 席位使用用量点数计费。需确认席位类型。",
      claudeOrg: "模型访问与额外计费遵循组织管理员的设置。",
      claudePicker: "在 Claude Code /model 中查看当前账户的模型。Fable 5.2 需要 2.1.321 或更高版本。",
      agyBase: "Antigravity：Gemini 4 Neon · Argon · Hydrogen、Gemini 3.8 Flash、Claude Sonnet 5.5 · Opus 5.5 (Thinking)",
      agyConditions: "Neon、Argon、Orchestrator：付费 Pro 或符合条件的高端设备试用。Hydrogen、Claude 5.5：仅付费 Pro。",
      agyLegacy: "GPT-OSS-120b 和试用版 Claude 4.6 计划于 2026-11-30 移除。",
      agyPicker: "在 Antigravity 应用模型选择器或 agy models 中查看当前账户的模型。",
    },
  };

  function lookup(input = {}, locale = "ko") {
    if (!input || typeof input !== "object" || Array.isArray(input)) return null;
    if ([input.provider, input.planType, input.orgType, input.planName, input.seatType]
      .some((value) => value != null && (typeof value !== "string" || value.length > 160 || /[\x00-\x1f\x7f]/.test(value)))) return null;
    const provider = normalized(input.provider);
    const planType = normalized(input.planType);
    const orgType = normalized(input.orgType);
    const planName = normalized(input.planName);
    const seatType = normalized(input.seatType);
    const t = TEXT[localeKey(locale)];
    let lines;
    let sourceName;

    if (["agy", "antigravity"].includes(provider)) {
      if ((planName || planType) !== "google ai pro") return null;
      sourceName = "Google Antigravity";
      lines = [t.agyBase, t.agyConditions, t.agyLegacy, t.agyPicker];
    } else if (provider === "claude") {
      const type = orgType || planType || planName;
      if (["claude_team", "team"].includes(type)) {
        lines = [t.claudeBase, seatType === "premium" ? t.fableIncluded
          : seatType === "standard" ? t.fableCredits : t.fableTeamUnknown, t.claudeOrg, t.claudePicker];
      } else if (["claude_max", "max", "max 5x", "claude_ultra", "ultra"].includes(type)) {
        lines = [t.claudeBase, t.fableIncluded, t.claudePicker];
      } else if (["claude_pro", "pro"].includes(type)) {
        lines = [t.claudeBase, t.fableCredits, t.claudePicker];
      } else return null;
      sourceName = "Anthropic";
    } else if (provider === "codex") {
      const type = planType || planName;
      const business = ["team", "business"].includes(type) || (!type && ["standard", "premium"].includes(seatType));
      if (!business && !["plus", "pro", "prolite", "promax", "pro 100", "pro 200", "pro 500"].includes(type)) return null;
      lines = [t.codexBase, t.codexVersion];
      if (business) lines.push(t.codexBusiness);
      lines.push(t.codexPicker);
      sourceName = "OpenAI";
    } else return null;

    return { title: t.title, lines, source: t.source(sourceName) };
  }

  return { lookup };
});
