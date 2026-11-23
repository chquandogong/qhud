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
