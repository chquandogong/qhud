// Connection evidence and quota freshness are separate facts. This module
// formats both without making requests or exposing raw provider errors.
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root && root.document) root.QhudProviderHealth = api;
})(typeof window === "undefined" ? null : window, function () {
  "use strict";

  const PROVIDERS = ["codex", "claude", "agy"];
  const NAMES = { codex: "Codex", claude: "Claude", agy: "Antigravity" };
  const TEXT = {
    ko: {
      ready: "조회 준비", connected: "조회 성공", login_required: "로그인 필요", refresh_needed: "인증 갱신 대기",
      credential_expired: "인증 만료", cli_missing: "연결 도구 없음", not_running: "앱 미실행",
      unsupported: "연결 미지원", error: "조회 실패", unknown: "상태 미확인", loading: "조회 중…",
      readyDetail: "로컬 연결 정보를 찾았습니다. ⟳를 눌러 현재 사용량을 확인하세요.",
      refresh_neededDetail: "저장된 액세스 인증이 만료됐습니다. ⟳를 누르면 Codex가 자체 로그인 갱신을 시도합니다.",
      connectedDetail: "마지막 사용량 요청에 응답했습니다. 수치는 조회 시점의 값입니다.",
      login_requiredDetail: "사용량 조회에 필요한 로그인을 확인할 수 없습니다.",
      credential_expiredDetail: "qhud가 사용하는 저장 인증이 만료됐습니다. 앱의 로그인 상태와 다를 수 있습니다.",
      cli_missingDetail: "사용량 조회에 필요한 연결 도구를 찾지 못했습니다.",
      not_runningDetail: "Antigravity를 실행하고 로그인한 뒤 다시 조회하세요.",
      unsupportedDetail: "현재 앱에서 읽을 수 있는 사용량 연결을 찾지 못했습니다.",
      errorDetail: "사용량을 갱신하지 못했습니다. 저장된 수치는 현재 값으로 간주하지 마세요.",
      rate_limitedDetail: "공급자가 조회 요청을 제한했습니다. 잠시 기다린 뒤 다시 조회하세요.",
      network_errorDetail: "공급자에 연결하지 못했습니다. 네트워크를 확인한 뒤 다시 조회하세요.",
      invalid_responseDetail: "응답에서 유효한 사용량을 읽지 못했습니다. 이전 저장값을 유지합니다.",
      refresh_unavailableDetail: "현재 연결 경로로 사용량을 조회하지 못했습니다.",
      unknownDetail: "연결 상태가 아직 확인되지 않았습니다. ⟳를 눌러 확인하세요.",
      loadingDetail: "현재 사용량을 요청하고 있습니다. 기존 수치는 마지막 조회값입니다.",
      noData: "사용량 없음", noWindows: "사용 한도 미제공", saved: "저장값", savedAccount: "마지막 조회 계정", ageUnknown: "저장 시각 미상",
      lastSuccess: "마지막 성공", lastAttempt: "마지막 시도", expires: "저장 인증 만료", expiryUnknown: "인증 만료 시각 미제공",
      source: "연결 경로", refresh: "사용량 새로고침", details: "연결 상세", noSuccess: "성공한 조회 없음",
      resetCreditsMissing: "초기화권 정보 없음", resetCreditsHint: "조회 시점의 남은 초기화권",
      wrong_account: "다른 계정", wrong_scope: "다른 조직", personal: "개인", team: "팀",
      profileStatus: "등록 연결 상태",
      profileLoginRequired: "qhud 연결 필요",
      seatWebVerified: "웹 확인", seatLabel: "좌석 유형", seatVerifiedAt: "확인 시각",
      seatSource: "출처: 관리자 Members 페이지", seatHistorical: "웹에서 확인한 기록입니다. 자동으로 최신 상태를 확인하지 않습니다.",
      organizationPlan: "조직 플랜",
      profileConnect: "로컬 계정 연결 스크립트에서 이 계정과 조직에 로그인한 뒤 ⟳를 누르세요.",
      profileWrongAccount: "저장된 로그인이 등록한 이메일과 다릅니다. 이 항목에는 해당 사용량을 표시하지 않습니다.",
      profileWrongScope: "저장된 로그인의 조직이 등록한 개인/팀 구분과 다릅니다. 이 항목에는 해당 사용량을 표시하지 않습니다.",
      justNow: "방금", minute: "분 전", hour: "시간 전", day: "일 전",
      defaultAction: "⟳로 다시 조회하세요.", loginAction: "연결된 CLI 또는 앱에서 로그인한 뒤 ⟳를 누르세요.",
      claudeLoginAction: "Claude Code CLI에서 로그인한 뒤 ⟳를 누르세요. 데스크톱 앱 로그인만으로 갱신되지는 않습니다.",
      codexLoginAction: "Codex 앱 또는 CLI의 로그인 상태를 확인한 뒤 ⟳를 누르세요.",
      cliAction: "Codex CLI를 설치하거나 QHUD_CODEX_BIN에 실행 파일을 지정하세요.",
      terminalEmpty: "연결된 터미널 세션 없음 · 계정 사용량은 위에서 조회하세요.",
    },
    en: {
      ready: "Ready to fetch", connected: "Fetch succeeded", login_required: "Sign-in needed", refresh_needed: "Auth renewal needed",
      credential_expired: "Auth expired", cli_missing: "Connector missing", not_running: "App not running",
      unsupported: "Unsupported", error: "Fetch failed", unknown: "Not checked", loading: "Fetching…",
      readyDetail: "Local connection information was found. Press ⟳ to check current usage.",
      refresh_neededDetail: "The stored access credential expired. Press ⟳ to let Codex attempt its own sign-in renewal.",
      connectedDetail: "The last usage request succeeded. Numbers describe that fetch, not a live stream.",
      login_requiredDetail: "The sign-in needed to read usage could not be verified.",
      credential_expiredDetail: "The stored credential used by qhud has expired. The desktop app may use a different sign-in.",
      cli_missingDetail: "The connector needed to read usage could not be found.",
      not_runningDetail: "Start Antigravity, sign in, then refresh.",
      unsupportedDetail: "No supported usage connection was found for the current app.",
      errorDetail: "Usage could not be refreshed. Stored numbers are not current readings.",
      rate_limitedDetail: "The provider limited usage requests. Wait before retrying.",
      network_errorDetail: "The provider could not be reached. Check the network and retry.",
      invalid_responseDetail: "The response did not contain readable usage. Previous stored values are retained.",
      refresh_unavailableDetail: "Usage could not be fetched through the current connection.",
      unknownDetail: "Connection status has not been checked. Press ⟳ to check.",
      loadingDetail: "Requesting usage. Existing numbers are from the previous fetch.",
      noData: "No usage data", noWindows: "No quota supplied", saved: "Saved", savedAccount: "Last fetched account", ageUnknown: "Age unknown",
      lastSuccess: "Last success", lastAttempt: "Last attempt", expires: "Stored auth expires", expiryUnknown: "Auth expiry unavailable",
      source: "Connection", refresh: "Refresh usage", details: "Connection details", noSuccess: "No successful fetch",
      resetCreditsMissing: "Reset credits unavailable", resetCreditsHint: "Reset credits remaining at the time of the fetch",
      wrong_account: "Different account", wrong_scope: "Different organization", personal: "Personal", team: "Team",
      profileStatus: "Registered connection",
      profileLoginRequired: "Connect to qhud",
      seatWebVerified: "Web verified", seatLabel: "Seat type", seatVerifiedAt: "Verified at",
      seatSource: "Source: admin Members page", seatHistorical: "A dated web observation; it is not refreshed automatically.",
      organizationPlan: "Organization plan",
      profileConnect: "Use the local account connection script to sign in to this account and organization, then press ⟳.",
      profileWrongAccount: "The stored login uses a different email. Its usage is not assigned to this entry.",
      profileWrongScope: "The stored login belongs to a different personal/team organization. Its usage is not assigned to this entry.",
      justNow: "just now", minute: "m ago", hour: "h ago", day: "d ago",
      defaultAction: "Press ⟳ to retry.", loginAction: "Sign in through the connected CLI or app, then press ⟳.",
      claudeLoginAction: "Sign in through Claude Code CLI, then press ⟳. Desktop app sign-in alone does not update this credential.",
      codexLoginAction: "Check sign-in in the Codex app or CLI, then press ⟳.",
      cliAction: "Install Codex CLI or set QHUD_CODEX_BIN to its executable.",
      terminalEmpty: "No connected terminal sessions · Refresh account usage above.",
    },
    zh: {
      ready: "可查询", connected: "查询成功", login_required: "需要登录", refresh_needed: "认证待更新",
      credential_expired: "认证已过期", cli_missing: "缺少连接工具", not_running: "应用未运行",
      unsupported: "暂不支持", error: "查询失败", unknown: "尚未检查", loading: "查询中…",
      readyDetail: "已找到本地连接信息。点击 ⟳ 查询当前用量。",
      refresh_neededDetail: "已存访问凭据已过期。点击 ⟳ 让 Codex 尝试自行更新登录凭据。",
      connectedDetail: "上次用量请求成功。数值对应查询时刻，并非实时更新。",
      login_requiredDetail: "无法确认查询用量所需的登录状态。",
      credential_expiredDetail: "qhud 使用的已存凭据已过期，可能与桌面应用的登录状态不同。",
      cli_missingDetail: "未找到查询用量所需的连接工具。",
      not_runningDetail: "启动 Antigravity 并登录后重新查询。",
      unsupportedDetail: "未找到当前应用支持的用量连接。",
      errorDetail: "未能更新用量。已保存的数值不是当前读数。",
      rate_limitedDetail: "供应商限制了用量查询请求。请稍后重试。",
      network_errorDetail: "无法连接供应商。请检查网络后重试。",
      invalid_responseDetail: "无法从响应中读取有效用量，保留之前保存的数值。",
      refresh_unavailableDetail: "当前连接方式无法查询用量。",
      unknownDetail: "尚未确认连接状态。点击 ⟳ 检查。",
      loadingDetail: "正在请求用量。现有数值来自上次查询。",
      noData: "无用量数据", noWindows: "未提供配额", saved: "已保存", savedAccount: "上次查询账户", ageUnknown: "保存时间未知",
      lastSuccess: "上次成功", lastAttempt: "上次尝试", expires: "已存认证到期", expiryUnknown: "未提供认证到期时间",
      source: "连接方式", refresh: "刷新用量", details: "连接详情", noSuccess: "尚无成功查询",
      resetCreditsMissing: "未提供重置券信息", resetCreditsHint: "查询时剩余的重置券",
      wrong_account: "账户不符", wrong_scope: "组织不符", personal: "个人", team: "团队",
      profileStatus: "已登记连接状态",
      profileLoginRequired: "需要连接 qhud",
      seatWebVerified: "网页确认", seatLabel: "席位类型", seatVerifiedAt: "确认时间",
      seatSource: "来源：管理员 Members 页面", seatHistorical: "这是网页确认记录，不会自动检查最新状态。",
      organizationPlan: "组织套餐",
      profileConnect: "使用本地账户连接脚本登录此账户与组织，然后点击 ⟳。",
      profileWrongAccount: "已存登录的邮箱与登记邮箱不同，不会将其用量归入此项。",
      profileWrongScope: "已存登录的个人/团队组织与登记组织不同，不会将其用量归入此项。",
      justNow: "刚刚", minute: "分钟前", hour: "小时前", day: "天前",
      defaultAction: "点击 ⟳ 重试。", loginAction: "通过已连接的 CLI 或应用登录后，点击 ⟳。",
      claudeLoginAction: "在 Claude Code CLI 中登录后点击 ⟳。仅登录桌面应用不会更新此凭据。",
      codexLoginAction: "检查 Codex 应用或 CLI 的登录状态后点击 ⟳。",
      cliAction: "安装 Codex CLI，或通过 QHUD_CODEX_BIN 指定其可执行文件。",
      terminalEmpty: "无已连接的终端会话 · 请在上方刷新账户用量。",
    },
  };
  const STATES = ["ready", "connected", "login_required", "credential_expired", "cli_missing", "not_running", "unsupported", "error", "unknown"];
  const timestamp = (value) => typeof value === "number" && Number.isFinite(value) && value > 0 ? value : null;
  const localeKey = (value) => String(value || "ko").startsWith("zh") ? "zh" : String(value || "ko").startsWith("en") ? "en" : "ko";
  const strings = (locale) => TEXT[localeKey(locale)];
  const validPercent = (value) => typeof value === "number" && Number.isFinite(value) && value >= 0 && value <= 100;
  const hasQuota = (q) => validPercent(q?.h5?.pct) || validPercent(q?.d7?.pct) || (q?.scoped || []).some((x) => validPercent(x?.pct));

  // A count is attached to its own account/workspace. A fresh response that
  // omits the field must not inherit an older (possibly spent) count.
  function resetCreditsView(row, freshUsage = null, locale = "ko") {
    const value = (freshUsage || row)?.reset_credits;
    const count = Number.isInteger(value) && value >= 0 && value <= 0xffffffff ? value : null;
    const t = strings(locale);
    const label = count == null ? null
      : localeKey(locale) === "ko" ? `초기화권 ${count}개`
        : localeKey(locale) === "zh" ? `重置券 ${count}张` : `Reset credits: ${count}`;
    return { count, label, detail: label ? `${label} · ${t.resetCreditsHint}` : t.resetCreditsMissing, hint: t.resetCreditsHint };
  }

  function configuredConnectionView(profile, locale = "ko") {
    if (!profile || profile.has_row || !PROVIDERS.includes(profile.provider) || typeof profile.key !== "string" || !profile.key) return null;
    const t = strings(locale);
    const state = ["login_required", "credential_expired", "wrong_account", "wrong_scope", "ready"].includes(profile.state) ? profile.state : "unknown";
    const detail = [];
    if (state === "wrong_account") detail.push(t.profileWrongAccount);
    else if (state === "wrong_scope") detail.push(t.profileWrongScope);
    if (state !== "ready") detail.push(t.profileConnect);
    else detail.push(t.readyDetail);
    const expires = timestamp(profile.expires_at_ms);
    if (expires) detail.push(`${t.expires}: ${new Date(expires).toLocaleString(localeKey(locale) === "ko" ? "ko-KR" : localeKey(locale) === "zh" ? "zh-CN" : "en-US")}`);
    return { key: profile.key, provider: profile.provider, email: profile.email || profile.label || "", scope: ["personal", "team"].includes(profile.scope) ? t[profile.scope] : "", state, label: state === "login_required" ? t.profileLoginRequired : t[state], detail: detail.join("\n") };
  }

  function seatAnnotationView(annotation, locale = "ko") {
    if (!annotation || !["standard", "premium"].includes(annotation.seat_type) ||
        !Number.isSafeInteger(annotation.verified_at_ms) || annotation.verified_at_ms <= 0 ||
        annotation.verified_at_ms > Date.now() + 60_000) return null;
    const t = strings(locale);
    const seat = annotation.seat_type === "standard" ? "Standard" : "Premium";
    const label = `${seat} · ${t.seatWebVerified}`;
    const verified = new Date(annotation.verified_at_ms).toLocaleString(localeKey(locale) === "ko" ? "ko-KR" : localeKey(locale) === "zh" ? "zh-CN" : "en-US");
    return { label, workspaceName: annotation.workspace_name, detail: `${t.seatLabel}: ${label}\n${t.seatSource}\n${t.seatVerifiedAt}: ${verified}\n${t.seatHistorical}` };
  }

  function configuredConnectionDetail(profile, locale = "ko") {
    const model = configuredConnectionView({ ...profile, has_row: false }, locale);
    if (!model) return "";
    const t = strings(locale);
    return `${t.profileStatus}${model.scope ? ` (${model.scope})` : ""}: ${model.label}\n${model.detail}`;
  }

  function age(at, now, locale) {
    const t = strings(locale);
    if (!timestamp(at) || at > now + 60_000) return t.ageUnknown;
    const minutes = Math.max(0, Math.floor((now - at) / 60_000));
    if (minutes < 1) return t.justNow;
    if (minutes < 60) return `${minutes}${t.minute}`;
    if (minutes < 1440) return `${Math.floor(minutes / 60)}${t.hour}`;
    return `${Math.floor(minutes / 1440)}${t.day}`;
  }

  // Legacy commands reject with strings; classify without displaying payloads,
  // URLs, filesystem paths, or account information contained in those strings.
  function errorState(error) {
    const value = String(error || "");
    if (/expired|expiry|credential_expired/i.test(value)) return "credential_expired";
    if (/401|unauthori[sz]ed|sign.?in|login|credential.*missing|no.*credential/i.test(value)) return "login_required";
    if (/no running|not running|start agy|start antigravity|no.*listener/i.test(value)) return "not_running";
    if (/codex.*not found|executable.*not found|QHUD_CODEX_BIN|cli_missing/i.test(value)) return "cli_missing";
    if (/unsupported|not supported/i.test(value)) return "unsupported";
    return "error";
  }

  function view(health = {}, options = {}) {
    const { provider = health.provider, now = Date.now(), locale = "ko", fetch = {}, quotas = [] } = options;
    const t = strings(locale);
    let state = STATES.includes(health.state) ? health.state : "unknown";
    let lastSuccess = timestamp(health.last_success_at_ms);
    const attempt = timestamp(health.last_attempt_at_ms);
    // A poll that arrives while the explicit request runs cannot erase its spinner.
    const localFinished = fetch.finishedAt || fetch.at || 0;
    const checked = timestamp(health.checked_at_ms);
    const localNewest = (!attempt || localFinished >= attempt) && (!checked || localFinished > checked);
    if (fetch.state === "loading") state = "loading";
    else if (localNewest && fetch.state === "error") state = errorState(fetch.error);
    else if (localNewest && fetch.state === "done" && timestamp(fetch.at)) {
      state = "connected";
      lastSuccess = Math.max(lastSuccess || 0, fetch.at);
    }
    if (state === "ready" && health.code === "codex_refresh_needed") state = "refresh_needed";
    const expires = timestamp(health.expires_at_ms);
    if (["ready", "connected"].includes(state) && expires && expires <= now && (!lastSuccess || lastSuccess <= expires)) state = "credential_expired";
    const usage = quotas.filter((q) => q.provider === provider && hasQuota(q));
    const savedAt = usage.reduce((at, q) => Math.max(at, timestamp(q.cache_fetched_at_ms) || 0), 0) || null;
    // A provider-wide success may belong to a different configured account.
    // Never move an existing snapshot's clock forward to that success time.
    const quotaAt = savedAt || lastSuccess;
    const summary = usage.length
      ? `${usage.some((q) => q.origin === "saved-account") ? t.savedAccount : t.saved} · ${quotaAt ? age(quotaAt, now, locale) : t.ageUnknown}`
      : state === "connected" ? t.noWindows : t.noData;
    const errorCode = { network: "network_error", unavailable: "refresh_unavailable" }[health.error_kind] || health.error_kind;
    const detail = [(state === "error" && (t[`${errorCode}Detail`] || t[`${health.code}Detail`])) || t[`${state}Detail`] || t.unknownDetail];
    const exactDate = (ms) => new Date(ms).toLocaleString(localeKey(locale) === "zh" ? "zh-CN" : localeKey(locale) === "ko" ? "ko-KR" : "en-US");
    detail.push(lastSuccess ? `${t.lastSuccess}: ${exactDate(lastSuccess)}` : t.noSuccess);
    if (attempt) detail.push(`${t.lastAttempt}: ${exactDate(attempt)}`);
    detail.push(expires ? `${t.expires}: ${exactDate(expires)}` : t.expiryUnknown);
    const sourceNames = {
      codex_app_server: "Codex app-server", claude_code: "Claude Code CLI",
      claude_statusline: "Claude Code statusline", agy_loopback: "Antigravity local RPC",
      antigravity_loopback: "Antigravity local RPC", antigravity_desktop: "Antigravity Desktop",
      "antigravity-desktop": "Antigravity Desktop", "agy-cli": "Antigravity CLI",
    };
    if (sourceNames[health.source]) detail.push(`${t.source}: ${sourceNames[health.source]}`);
    if (["login_required", "credential_expired"].includes(state)) detail.push(provider === "claude" ? t.claudeLoginAction : provider === "codex" ? t.codexLoginAction : t.loginAction);
    else if (state === "cli_missing") detail.push(t.cliAction);
    else if (state === "error") detail.push(t.defaultAction);
    return {
      provider, name: NAMES[provider] || provider, state, label: t[state], summary,
      detail: detail.join("\n"), lastSuccess, quotaAt,
      stale: usage.length > 0 && (state !== "connected" || !quotaAt || now - quotaAt > 5 * 60_000),
      tone: state === "connected" ? "good" : ["credential_expired", "login_required", "error"].includes(state) ? "warn" : "muted",
      busy: state === "loading", refreshLabel: `${NAMES[provider] || provider} · ${t.refresh}`, detailsLabel: t.details,
    };
  }

  return { PROVIDERS, NAMES, strings, validPercent, hasQuota, resetCreditsView, configuredConnectionView, configuredConnectionDetail, seatAnnotationView, age, errorState, view };
});
