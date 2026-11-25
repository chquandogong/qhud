const test = require("node:test");
const assert = require("node:assert/strict");
const health = require("../ui/provider-health.js");

const NOW = Date.UTC(2026, 10, 17, 6);
const quota = (at = NOW - 60_000) => ({ provider: "codex", origin: "fetched", h5: { pct: 0 }, cache_fetched_at_ms: at });

test("reset credits distinguish unknown, zero, and a positive remaining count", () => {
  const missing = health.resetCreditsView({});
  assert.equal(missing.label, null);
  assert.equal(missing.detail, "초기화권 정보 없음");
  assert.equal(health.resetCreditsView({ reset_credits: 0 }).label, "초기화권 0개");
  assert.equal(health.resetCreditsView({ reset_credits: 3 }).label, "초기화권 3개");
  assert.equal(health.resetCreditsView({ reset_credits: 3 }, null, "en").label, "Reset credits: 3");
  assert.equal(health.resetCreditsView({ reset_credits: 3 }, null, "zh-CN").label, "重置券 3张");
  for (const value of [null, undefined, -1, NaN, Infinity, 0.5, "3", 4294967296]) {
    assert.equal(health.resetCreditsView({ reset_credits: value }).label, null);
  }
});

test("reset credits stay account scoped and missing fresh data cannot revive an old count", () => {
  const personal = { account: { account_id: "same-user", org_id: "personal" }, reset_credits: 3 };
  const team = { account: { account_id: "same-user", org_id: "team" }, reset_credits: 0 };
  const rows = [personal, team, { account: { account_id: "other-user" } }];
  assert.deepEqual(rows.map((row) => health.resetCreditsView(row).count), [3, 0, null]);
  assert.equal(health.resetCreditsView(personal, { reset_credits: 1 }).count, 1);
  assert.equal(health.resetCreditsView(personal, { reset_credits: null }).count, null);
  assert.equal(health.resetCreditsView(personal, {}).count, null);
  assert.equal(health.resetCreditsView(team).count, 0);
});

const member = {
  provider: "claude", origin: "fetched", cache_fetched_at_ms: NOW - 60_000,
  account: { email: "member@example.com", account_id: "person", org_id: "org-team", org_type: "claude_team", config_dir: "team-profile" },
  h5: { pct: 18 }, d7: { pct: 9 },
};
const observation = {
  email: "MEMBER@example.com", org_id: "org-team", observed_at_ms: NOW - 5000,
  offers: [{ kind: "full", count: 1, expires_at_ms: NOW + 1000 }, { kind: "fable", count: 2, expires_at_ms: NOW + 5000 }],
};
const resetView = (opts = {}) => health.resetCreditsView(opts.row || member, opts.live, opts.locale,
  opts.notes || [observation], opts.now ?? NOW);

test("a saved Claude member row discloses dated counts, kinds and local expiry in each language", () => {
  for (const [locale, list] of [["ko-KR", ["초기화권 3개 · 확인값", "전체 초기화 1개", "Fable 초기화 2개"]],
    ["zh-CN", ["重置券 3张 · 上次确认", "全部限额 1张", "Fable 2张"]],
    ["en-US", ["Reset credits: 3 · last observed", "Full reset × 1", "Fable × 2"]]]) {
    const view = resetView({ locale });
    assert.equal(view.count, 3);
    assert.equal(view.label, list[0]);
    assert.equal(view.observed, true);
    assert.equal(view.nextExpiry, NOW + 1000);
    for (const item of list.slice(1)) assert.ok(view.hint.includes(item));
    const fmt = new Intl.DateTimeFormat(locale, {
      year: "numeric", month: "short", day: "numeric", hour: "numeric", minute: "2-digit", timeZoneName: "short",
    });
    for (const date of [observation.observed_at_ms, ...observation.offers.map((offer) => offer.expires_at_ms)]) {
      assert.ok(view.hint.includes(fmt.format(date)));
    }
  }
  const note = { ...observation, offers: [
    { kind: "five_hour", count: 1, expires_at_ms: NOW + 60_000 },
    { kind: "weekly", count: 1, expires_at_ms: NOW + 3_600_000 },
  ] };
  assert.match(resetView({ notes: [note] }).hint, /5시간 한도 초기화 1개[\s\S]*주간 한도 초기화 1개/);
});

test("Claude reset notes require matching provider, member email and exact org, including inactive rows", () => {
  assert.equal(resetView({ row: { ...member, origin: "saved-account" } }).count, 3);
  for (const account of [
    { ...member.account, org_id: "org-personal", org_type: "claude_free" },
    { ...member.account, org_id: "ORG-TEAM" }, { ...member.account, email: "colleague@example.com" },
    { email: member.account.email }, { org_id: observation.org_id },
  ]) assert.equal(resetView({ row: { ...member, account } }).label, null);
  assert.equal(resetView({ row: { ...member, provider: "codex" } }).label, null);
  assert.equal(resetView({ notes: [{ ...observation, observed_at_ms: NOW + 1 }] }).label, null);
});

test("live provider reset counts take precedence, including an explicitly spent balance", () => {
  for (const count of [0, 1]) {
    const actual = resetView({ live: { reset_credits: count } });
    assert.equal(actual.count, count);
    assert.equal(actual.observed, undefined);
    assert.doesNotMatch(actual.detail, /확인값|Fable/);
  }
  const freshWithoutCount = resetView({ row: { ...member, reset_credits: 4 }, live: { five_hour: null } });
  assert.equal(freshWithoutCount.label, "초기화권 3개 · 확인값");
  assert.match(freshWithoutCount.hint, /확인 후 사용한 초기화권/);
});

test("expiry changes the observed total without claiming a live zero or reviving earlier offers", () => {
  const older = { ...observation, observed_at_ms: NOW - 10_000,
    offers: [{ kind: "weekly", count: 10, expires_at_ms: NOW + 50_000 }] };
  const notes = [observation, older];
  const partial = resetView({ notes, now: NOW + 1000 });
  assert.equal(partial.label, "초기화권 2개 · 확인값");
  assert.equal(partial.nextExpiry, NOW + 5000);
  assert.match(partial.hint, /전체 초기화 1개[^\n]*만료됨/);
  const expired = resetView({ notes, now: NOW + 5000 });
  assert.equal(expired.count, null);
  assert.equal(expired.nextExpiry, null);
  assert.equal(expired.label, "확인한 초기화권 만료");
  const zero = resetView({ notes: [{ ...observation, offers: [], observed_at_ms: NOW }, ...notes] });
  assert.equal(zero.label, "초기화권 0개 · 확인값");
  assert.equal(zero.count, 0);
  assert.equal(resetView({ notes: [observation, { ...observation, offers: [] }] }).label, null);
  assert.equal(resetView({ notes: [{ ...observation, offers: [{ kind: "credit", count: 4, expires_at_ms: NOW + 1 }] }] }).label, null);
});

test("local credentials establish readiness, never a successful live connection", () => {
  const view = health.view({ provider: "codex", state: "ready", cli_available: true }, { now: NOW });
  assert.equal(view.state, "ready");
  assert.equal(view.label, "조회 준비");
  assert.equal(view.lastSuccess, null);
  assert.equal(view.summary, "사용량 없음");
  assert.notEqual(view.tone, "good");
});

test("all providers stay discoverable with no accounts or health payload", () => {
  assert.deepEqual(health.PROVIDERS, ["codex", "claude", "agy"]);
  for (const provider of health.PROVIDERS) {
    const view = health.view({}, { provider, now: NOW });
    assert.equal(view.state, "unknown");
    assert.equal(view.summary, "사용량 없음");
    assert.ok(view.refreshLabel.includes(health.NAMES[provider]));
    assert.equal(view.lastSuccess, null);
  }
  assert.equal(health.view({ state: "future_state" }, { now: NOW }).state, "unknown");
});

test("expired credentials remain explicit while dated quota readings are retained", () => {
  const saved = NOW - 7 * 86400_000;
  const view = health.view({ provider: "claude", state: "ready", expires_at_ms: NOW - 86400_000 }, {
    now: NOW, quotas: [{ ...quota(saved), provider: "claude" }],
  });
  assert.equal(view.state, "credential_expired");
  assert.equal(view.summary, "저장값 · 7일 전");
  assert.equal(view.stale, true);
  assert.match(view.detail, /데스크톱 앱 로그인만으로 갱신되지는 않습니다/);
  assert.doesNotMatch(view.summary, /0%/);
});

test("Codex can renew expired access credentials through its own connector", () => {
  const view = health.view({ provider: "codex", state: "ready", code: "codex_refresh_needed", expires_at_ms: NOW - 1000 }, { now: NOW });
  assert.equal(view.state, "refresh_needed");
  assert.equal(view.label, "인증 갱신 대기");
  assert.match(view.detail, /Codex가 자체 로그인 갱신/);
  assert.doesNotMatch(view.detail, /로그인한 뒤/);
});

test("structured errors distinguish network failure and provider rate limiting", () => {
  const network = health.view({ provider: "codex", state: "error", error_kind: "network", code: "network_error" }, { now: NOW });
  const limited = health.view({ provider: "claude", state: "error", code: "rate_limited" }, { now: NOW });
  assert.match(network.detail, /네트워크/);
  assert.match(limited.detail, /요청을 제한/);
});

test("a newer provider-level success cannot make an older account snapshot fresh", () => {
  const savedAt = NOW - 3600_000;
  const view = health.view({ provider: "codex", state: "connected", last_success_at_ms: NOW }, { now: NOW, quotas: [quota(savedAt)] });
  assert.equal(view.lastSuccess, NOW);
  assert.equal(view.quotaAt, savedAt);
  assert.equal(view.summary, "저장값 · 1시간 전");
  assert.equal(view.stale, true);
});

test("refresh failure preserves snapshot age and does not advance last success", () => {
  const last = NOW - 3600_000;
  const view = health.view({ provider: "codex", state: "connected", last_success_at_ms: last }, {
    now: NOW, quotas: [quota(last)], fetch: { state: "error", error: "request failed: 503", finishedAt: NOW },
  });
  assert.equal(view.state, "error");
  assert.equal(view.lastSuccess, last);
  assert.equal(view.quotaAt, last);
  assert.equal(view.summary, "저장값 · 1시간 전");
  assert.equal(view.stale, true);
});

test("a polling update cannot erase an in-flight request, but a newer backend outcome replaces an old local error", () => {
  const loading = health.view({ provider: "codex", state: "connected", last_attempt_at_ms: NOW }, {
    now: NOW, fetch: { state: "loading" },
  });
  assert.equal(loading.state, "loading");
  assert.equal(loading.busy, true);
  const recovered = health.view({ provider: "codex", state: "connected", last_attempt_at_ms: NOW, last_success_at_ms: NOW }, {
    now: NOW, quotas: [quota(NOW)], fetch: { state: "error", error: "401", finishedAt: NOW - 1000 },
  });
  assert.equal(recovered.state, "connected");
  assert.equal(recovered.stale, false);
  const stopped = health.view({ provider: "agy", state: "not_running", checked_at_ms: NOW, last_attempt_at_ms: NOW - 10_000, last_success_at_ms: NOW - 9000 }, {
    now: NOW, fetch: { state: "done", at: NOW - 9000, finishedAt: NOW - 8000 },
  });
  assert.equal(stopped.state, "not_running");
});

test("auth rejection and application absence are actionable without rendering raw errors", () => {
  const privateError = "401: token rejected; /private/account@example.com?token=secret";
  const view = health.view({ provider: "claude" }, { now: NOW, fetch: { state: "error", error: privateError, finishedAt: NOW } });
  assert.equal(view.state, "login_required");
  assert.doesNotMatch(JSON.stringify(view), /secret|account@example|private/);
  assert.equal(health.errorState("No running agy process has a 127.0.0.1 quota listener"), "not_running");
  assert.equal(health.errorState("Codex executable not found"), "cli_missing");
  assert.equal(health.errorState("Claude credential expired"), "credential_expired");
});

test("successful empty response is not a fabricated zero quota", () => {
  const view = health.view({ provider: "agy", state: "connected", last_success_at_ms: NOW }, { now: NOW });
  assert.equal(view.label, "조회 성공");
  assert.equal(view.summary, "사용 한도 미제공");
  assert.equal(view.stale, false);
  assert.doesNotMatch(JSON.stringify(view), /0%/);
  assert.equal(health.hasQuota(quota()), true); // measured zero is real data
  for (const value of [null, undefined, NaN, Infinity, -1, "0", 101]) {
    assert.equal(health.hasQuota({ h5: { pct: value } }), false);
  }
});

const personalFree = {
  key: "example-personal", provider: "claude", email: "owner@example.com",
  scope: "personal", state: "wrong_scope", has_row: false,
  observed_plan: { email: "owner@example.com", plan: "free", verified_at_ms: Date.UTC(2025, 5, 1) },
};

test("a web-confirmed Claude Free personal row has a dated plan and no usage", () => {
  const view = health.configuredConnectionView(personalFree, "en");
  assert.equal(view.state, "usage_unavailable");
  assert.equal(view.label, "Usage unavailable");
  assert.equal(view.plan, "Free");
  assert.equal(view.scope, "Personal");
  assert.match(view.detail, /2025/);
  assert.doesNotMatch(JSON.stringify(view), /0%|reset_credits/);
  assert.equal(health.configuredConnectionView(personalFree, "ko").label, "사용량 미제공");
  assert.equal(health.configuredConnectionView({ ...personalFree, has_row: true }), null);
});

test("Claude Free details distinguish the registered connection from the web plan and its usage", () => {
  const expiresAt = Date.UTC(2025, 6, 12, 12);
  const labels = health.strings("en");
  for (const state of ["ready", "credential_expired", "wrong_scope", "login_required"]) {
    const view = health.configuredConnectionView({ ...personalFree, state, expires_at_ms: expiresAt }, "en");
    assert.equal(view.plan, "Free");
    assert.equal(view.label, labels.usageUnavailable);
    const status = state === "login_required" ? labels.profileLoginRequired :
      state === "credential_expired" ? labels.usageAuthExpired : labels[state];
    assert.ok(view.detail.includes(`${labels.profileStatus}: ${status}`));
    if (state === "ready" || state === "credential_expired") {
      assert.ok(view.detail.includes(`${labels.expires}: ${new Date(expiresAt).toLocaleString("en-US")}`));
    } else {
      assert.ok(!view.detail.includes(labels.expires));
    }
  }
});

test("a Free plan observation cannot replace another account, organization, or provider's status", () => {
  const observed = personalFree.observed_plan;
  for (const patch of [
    { provider: "codex" }, { scope: "team" }, { state: "wrong_account" },
    { observed_plan: { ...observed, email: "other@example.com" } },
    { observed_plan: { ...observed, plan: "pro" } },
    { observed_plan: { ...observed, verified_at_ms: -1 } },
    { observed_plan: { ...observed, verified_at_ms: Date.now() + 86400_000 } },
  ]) {
    const view = health.configuredConnectionView({ ...personalFree, ...patch });
    assert.equal(view.state, patch.state || personalFree.state);
    assert.notEqual(view.plan, "Free");
  }
});

test("native Antigravity scoped quotas keep their fetched account provenance", () => {
  const view = health.view({ provider: "agy", state: "connected", last_success_at_ms: NOW }, {
    now: NOW, quotas: [{ provider: "agy", origin: "saved-account", cache_fetched_at_ms: NOW, scoped: [{ scope: "Gemini", pct: 25 }] }],
  });
  assert.equal(view.summary, "마지막 조회 계정 · 방금");
  assert.equal(view.quotaAt, NOW);
  assert.equal(view.stale, false);
});

test("unknown and future timestamps never become apparently fresh readings", () => {
  assert.equal(health.age(null, NOW), "저장 시각 미상");
  assert.equal(health.age(NOW + 3600_000, NOW), "저장 시각 미상");
  const view = health.view({ provider: "codex", state: "ready" }, { now: NOW, quotas: [{ ...quota(), cache_fetched_at_ms: null }] });
  assert.equal(view.summary, "저장값 · 저장 시각 미상");
  assert.equal(view.stale, true);
});

test("new status labels and guidance are available in Korean, English, and Chinese", () => {
  assert.equal(health.view({ state: "credential_expired" }, { locale: "en", now: NOW }).label, "Auth expired");
  assert.equal(health.view({ state: "credential_expired" }, { locale: "zh-CN", now: NOW }).label, "认证已过期");
  assert.equal(health.view({ state: "credential_expired" }, { now: NOW }).label, "인증 만료");
  for (const locale of ["ko", "en", "zh-CN"]) {
    for (const state of ["ready", "connected", "login_required", "credential_expired", "cli_missing", "not_running", "unsupported", "error", "unknown"]) {
      const view = health.view({ state }, { locale, now: NOW });
      assert.ok(view.label);
      assert.ok(view.detail);
      assert.doesNotMatch(view.detail, /undefined/);
    }
  }
});
