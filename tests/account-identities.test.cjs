const test = require("node:test");
const assert = require("node:assert/strict");
const identities = require("../ui/account-identities.js");
const health = require("../ui/provider-health.js");

const a = { email: "first@example.com", account_id: "shared-team", active: true, reset_credits: 3 };
const b = { email: "second@example.com", account_id: "shared-team", active: false, reset_credits: 0 };

test("account groups keep each login's personal, team, and unclassified rows in order", () => {
  const firstTeam = Object.freeze({ ...a, email: " FIRST@EXAMPLE.COM ", scope: "team" });
  const secondTeam = Object.freeze({ ...b, scope: "team" });
  const unknown = Object.freeze({ email: a.email, scope: "", account_id: "legacy-workspace" });
  const pendingPersonal = Object.freeze({ email: a.email, scope: "personal", state: "wrong_scope" });
  const otherTeam = Object.freeze({ email: a.email, scope: "team", account_id: "other-team" });
  const secondPersonal = Object.freeze({ email: b.email, scope: "personal", reset_credits: 0 });
  const input = Object.freeze([secondTeam, firstTeam, unknown, pendingPersonal, otherTeam, secondPersonal]);

  const groups = identities.groupAccountRows(input);
  assert.deepEqual(groups.map((group) => group.email), [b.email, a.email]);
  assert.deepEqual(groups[0].rows, [secondPersonal, secondTeam]);
  assert.deepEqual(groups[1].rows, [pendingPersonal, firstTeam, otherTeam, unknown]);
  assert.equal(groups[0].rows[1], secondTeam);
  assert.equal(groups[1].rows[0], pendingPersonal);
});

test("unidentified rows stay separate even when their visible labels or workspace IDs match", () => {
  const legacy = { label: "Example", account_id: a.account_id, scope: "team" };
  const other = { ...legacy, email: "  " };
  assert.deepEqual(identities.groupAccountRows([legacy, a, other]), [
    { email: "", rows: [legacy] },
    { email: a.email, rows: [a] },
    { email: "", rows: [other] },
  ]);
  assert.deepEqual(identities.groupAccountRows([]), []);
});

test("workspace scope follows the provider's wire type, including free or unrecognized plans", () => {
  const cases = [
    ["codex", "personal", ["free", "go", "plus", "prolite", "pro", "promax"]],
    ["codex", "team", ["team", "business", "enterprise"]],
    ["claude", "personal", ["claude_free", "claude_pro", "claude_max", "claude_ultra"]],
    ["claude", "team", ["claude_team", "claude_enterprise"]],
    ["codex", "", ["claude_team", "Example Team", "future_type", undefined]],
    ["claude", "", ["business", "Example Team", "future_type", undefined]],
    ["agy", "", ["pro", "team"]],
  ];
  for (const [provider, expected, types] of cases) {
    for (const type of types) assert.equal(identities.accountScope(provider, type), expected, `${provider} / ${type}`);
  }
});

test("a shared workspace ID never merges different login emails", () => {
  assert.notEqual(identities.workspaceKey(a), identities.workspaceKey(b));
  assert.notEqual(identities.accountKey(a), identities.accountKey(b));
  assert.equal(identities.workspaceKey(a), identities.workspaceKey({ ...a, email: " FIRST@example.com " }));
  assert.deepEqual(identities.distinctWorkspaces([a, b, { ...a }]), [a, b]);
});

test("active workspace overrides match login email and workspace together", () => {
  assert.equal(identities.activeWorkspace([a, b], a), a);
  assert.equal(identities.activeWorkspace([a, b], b), null);
  assert.equal(identities.activeWorkspace([a], {}), null);
  assert.equal(identities.activeWorkspace([a, b], { account_id: a.account_id }), null);
  const secondActive = { ...b, active: true };
  assert.equal(identities.activeWorkspace([a, secondActive], b), secondActive);
  assert.equal(identities.activeWorkspace([a, secondActive], { account_id: "shared-team" }), null);
  assert.equal(identities.activeWorkspace([a], { email: a.email, account_id: "personal" }), null);
});

test("a single legacy unknown-email workspace stays compatible without bridging known different logins", () => {
  const legacy = { account_id: "shared-team", active: true };
  assert.equal(identities.activeWorkspace([legacy], a), legacy);
  assert.equal(identities.activeWorkspace([a], { account_id: "shared-team" }), a);
  assert.equal(identities.activeWorkspace([legacy, { ...b, active: true }], a), null);
  assert.equal(identities.activeWorkspace([legacy], a, [a, b]), null);
  assert.equal(identities.activeWorkspace([legacy], b, [a, b]), null);
  assert.equal(identities.activeWorkspace([legacy], a, [a]), legacy);
});

test("newer backend snapshots replace in-session rows while preserving per-login counts", () => {
  const source = identities.codexSource({ state: "done", at: 100, rows: [a] }, { codex_fetched_at_ms: 200, codex_workspaces: [a, b] });
  assert.equal(source.stored, true);
  assert.equal(source.at, 200);
  assert.deepEqual(source.rows.map((row) => health.resetCreditsView(row).count), [3, 0]);
  const fresh = identities.codexSource({ state: "done", at: 300, rows: [a, b] }, { codex_fetched_at_ms: 200, codex_workspaces: [a] });
  assert.equal(fresh.stored, false);
  assert.equal(fresh.rows.length, 2);
});

test("Claude results must match config directory, account, and known organization", () => {
  const personal = { config_dir: "profile-personal", account_id: "one-user", org_id: "personal-org" };
  const result = { key: "profile-personal", account_id: "one-user", org_id: "personal-org", usage: { reset_credits: null } };
  assert.equal(identities.claudeUsageForAccount([result], personal), result);
  assert.equal(identities.claudeUsageForAccount([{ ...result, org_id: "team-org" }], personal), null);
  assert.equal(identities.claudeUsageForAccount([{ ...result, org_id: undefined }], personal), null);
  assert.equal(identities.claudeUsageForAccount([{ ...result, key: "profile-team" }], personal), null);
  assert.equal(identities.claudeUsageForAccount([{ ...result, account_id: "other-user" }], personal), null);
  assert.equal(identities.claudeUsageForAccount([result], { ...personal, org_id: undefined }), null);
  const legacyResult = { ...result, org_id: undefined };
  assert.equal(identities.claudeUsageForAccount([legacyResult], { ...personal, org_id: undefined }), legacyResult);
  assert.notEqual(identities.accountKey(personal), identities.accountKey({ ...personal, org_id: "team-org" }));
});

test("an older Antigravity button result cannot overwrite a newer account-owned plan snapshot", () => {
  const row = { account: { account_id: "first@example.com" }, cache_fetched_at_ms: 200, plan_name: null };
  const old = { account_id: "first@example.com", fetched_at_ms: 100, plan_name: "Google AI Pro" };
  assert.equal(identities.antigravityUsageForAccount(old, row), null);
  assert.equal(identities.antigravityUsageForAccount({ ...old, fetched_at_ms: undefined }, row), null);
  const fresh = { ...old, fetched_at_ms: 300 };
  assert.equal(identities.antigravityUsageForAccount(fresh, row), fresh);
  assert.equal(identities.antigravityUsageForAccount({ ...fresh, account_id: "second@example.com" }, row), null);
  assert.equal(identities.antigravityUsageForAccount(fresh, { ...row, account: {} }), null);
});

test("visible organization labels distinguish personal and team without inferring an unknown plan", () => {
  assert.equal(identities.organizationLabel("claude", { org_type: "claude_max" }), "개인");
  assert.equal(identities.organizationLabel("claude", { org_type: "claude_team", org: "Example" }), "팀 · Example");
  assert.equal(identities.organizationLabel("claude", { org_type: "future_type" }), "");
  assert.equal(identities.organizationLabel("codex", {}, "Example Team"), "Example Team");
});

test("configured accounts without verified rows remain numberless and mismatch states are explicit", () => {
  const profile = { provider: "claude", key: "first-team", email: "first@example.com", scope: "team", has_row: false, state: "login_required" };
  const pending = health.configuredConnectionView(profile);
  assert.equal(pending.scope, "팀");
  assert.equal(pending.label, "qhud 연결 필요");
  assert.doesNotMatch(JSON.stringify(pending), /0%|reset_credits|account_id/);
  assert.equal(health.configuredConnectionView({ ...profile, has_row: true }), null);
  assert.equal(health.configuredConnectionView({ ...profile, state: "wrong_account" }).label, "다른 계정");
  assert.equal(health.configuredConnectionView({ ...profile, state: "wrong_scope" }).label, "다른 조직");
  assert.equal(health.configuredConnectionView({ ...profile, state: "ready" }).label, "조회 준비");
  assert.equal(health.configuredConnectionView({ ...profile, provider: "other" }), null);
});

test("visible row connection details match email, workspace, and Claude organization exactly", () => {
  const profiles = [
    { provider: "codex", key: "first-team", has_row: true, email: a.email, account_id: a.account_id, scope: "team", state: "ready" },
    { provider: "codex", key: "second-team", has_row: true, email: b.email, account_id: b.account_id, scope: "team", state: "ready" },
  ];
  assert.deepEqual(identities.connectionsForAccount(profiles, "codex", a).map((p) => p.key), ["first-team"]);
  assert.deepEqual(identities.connectionsForAccount(profiles, "codex", b).map((p) => p.key), ["second-team"]);
  assert.deepEqual(identities.connectionsForAccount(profiles, "codex", { account_id: a.account_id }), []);
  const claude = { ...profiles[0], provider: "claude", account_id: "one-user", org_id: "team-org" };
  const account = { email: a.email, account_id: "one-user", org_id: "team-org" };
  assert.deepEqual(identities.connectionsForAccount([claude], "claude", account), [claude]);
  assert.deepEqual(identities.connectionsForAccount([claude], "claude", { ...account, org_id: "personal-org" }), []);
  assert.deepEqual(identities.connectionsForAccount([{ ...claude, org_id: undefined }], "claude", account), []);
});

test("verified profile detail includes its credential expiry without claiming a successful request", () => {
  const detail = health.configuredConnectionDetail({ provider: "claude", key: "first-team", has_row: true, state: "ready", scope: "team", expires_at_ms: Date.UTC(2026, 10, 18) });
  assert.match(detail, /등록 연결 상태 \(팀\): 조회 준비/);
  assert.match(detail, /로컬 연결 정보/);
  assert.match(detail, /저장 인증 만료/);
  assert.doesNotMatch(detail, /조회 성공/);
});

const seatTime = Date.UTC(2026, 0, 1);
const seatProfile = {
  provider: "codex", key: "second-team", email: b.email, scope: "team", state: "login_required", has_row: false,
  seat_annotation: { email: b.email, workspace_id: b.account_id, workspace_name: "Example Team", seat_type: "standard", verified_at_ms: seatTime },
};

test("web-verified seats remain visible for pending teams without claiming a usage connection", () => {
  const annotation = identities.seatAnnotationForProfile(seatProfile);
  const seat = health.seatAnnotationView(annotation);
  const connection = health.configuredConnectionView(seatProfile);
  assert.equal(seat.label, "Standard · 웹 확인");
  assert.equal(seat.workspaceName, "Example Team");
  assert.match(seat.detail, /관리자 Members 페이지/);
  assert.match(seat.detail, /2026/);
  assert.match(seat.detail, /자동으로 최신 상태를 확인하지 않습니다/);
  assert.equal(connection.label, "qhud 연결 필요");
  assert.equal(connection.state, "login_required");
  assert.doesNotMatch(JSON.stringify(seat), /0%|plan_type|조회 성공/);
});

test("web-verified seats match both email and workspace and never replace a provider plan", () => {
  const account = { ...b, plan_type: "business" };
  assert.equal(identities.seatAnnotationForAccount([seatProfile], "codex", account).seat_type, "standard");
  assert.equal(account.plan_type, "business");
  assert.equal(identities.seatAnnotationForAccount([seatProfile], "codex", a), null);
  assert.equal(identities.seatAnnotationForAccount([seatProfile], "codex", { ...b, account_id: "personal" }), null);
  assert.equal(identities.seatAnnotationForAccount([seatProfile], "codex", { account_id: b.account_id }), null);
  assert.equal(identities.seatAnnotationForAccount([seatProfile], "claude", b), null);
  assert.equal(identities.seatAnnotationForAccount([{ ...seatProfile, seat_annotation: undefined }], "codex", b), null);
  const newer = { ...seatProfile, seat_annotation: { ...seatProfile.seat_annotation, seat_type: "premium", verified_at_ms: seatTime + 1 } };
  assert.equal(identities.seatAnnotationForAccount([seatProfile, newer], "codex", b).seat_type, "premium");
});

test("malformed, mismatched, or future seat annotations cannot appear on a row", () => {
  const annotation = seatProfile.seat_annotation;
  for (const patch of [
    { email: a.email }, { workspace_id: "" }, { workspace_name: "bad\nname" },
    { seat_type: "business" }, { verified_at_ms: 0 }, { verified_at_ms: "2026" },
    { verified_at_ms: Date.now() + 120_000 },
  ]) {
    assert.equal(identities.seatAnnotationForProfile({ ...seatProfile, seat_annotation: { ...annotation, ...patch } }), null);
  }
  assert.equal(identities.seatAnnotationForProfile({ ...seatProfile, scope: "personal" }), null);
  assert.equal(identities.seatAnnotationForProfile({ ...seatProfile, provider: "claude" }), null);
  assert.equal(identities.seatAnnotationForProfile({ ...seatProfile, email: "" }), null);
  assert.equal(health.seatAnnotationView({ seat_type: "standard" }), null);
});

test("seat provenance and qhud-specific pending labels are translated without changing provider health", () => {
  assert.equal(health.configuredConnectionView(seatProfile, "en").label, "Connect to qhud");
  assert.equal(health.configuredConnectionView(seatProfile, "zh").label, "需要连接 qhud");
  assert.equal(health.strings("ko").login_required, "로그인 필요");
  const annotation = identities.seatAnnotationForProfile(seatProfile);
  assert.equal(health.seatAnnotationView(annotation, "en").label, "Standard · Web verified");
  assert.match(health.seatAnnotationView(annotation, "en").detail, /admin Members page/);
  assert.match(health.seatAnnotationView(annotation, "zh").detail, /不会自动检查最新状态/);
});
