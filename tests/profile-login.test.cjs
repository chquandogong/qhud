const { test } = require("node:test");
const assert = require("node:assert/strict");
const { controller, eligible } = require("../ui/profile-login.js");

const team = { key: "work-team", provider: "claude", email: "member@example.com", scope: "team",
  state: "credential_expired", can_reauthenticate: true, account_id: "member-a", org_id: "work-org" };

function harness(profile = team) {
  const calls = [];
  const stages = [];
  let current = profile, finish, fail;
  let result = [{ account_id: team.account_id, org_id: team.org_id, usage: { five_hour: { pct: 0 } } }];
  const logic = controller({
    lookup: (key) => current?.key === key ? current : null,
    authenticate: (key) => {
      calls.push(["login", key]);
      return new Promise((resolve, reject) => { finish = resolve; fail = reject; });
    },
    refresh: async (provider) => { calls.push(["refresh", provider]); return result; },
    changed: () => stages.push(logic.messageFor(profile)),
  });
  return { logic, calls, stages, finish: (verified = { ...profile, state: "ready" }) => finish(verified), fail: (e) => fail(e),
    setProfile: (p) => { current = p; }, setResults: (r) => { result = r; } };
}

test("only native-eligible registered profiles can start or repair login", () => {
  assert.ok(eligible(team));
  assert.ok(eligible({ ...team, state: "login_required" }));
  for (const state of ["wrong_account", "wrong_scope"]) {
    assert.ok(eligible({ ...team, state }));
    assert.equal(eligible({ ...team, state, can_reauthenticate: false }), false);
  }
  for (const patch of [{ can_reauthenticate: false }, { can_reauthenticate: undefined },
    { provider: "agy" }, { key: "../profile" }, { email: "" }, { scope: "other" },
    { state: "ready" },
    { scope: "personal", observed_plan: { email: team.email, plan: "free" } }]) {
    assert.equal(eligible({ ...team, ...patch }), false);
  }
});

test("repair accepts fresh quota for the verified Claude team while the prior wrong login is still on screen", async () => {
  const wrong = { ...team, state: "wrong_account", account_id: "other-member", org_id: "other-org" };
  for (const changed of [false, true]) {
    const h = harness(wrong);
    const task = h.logic.start(wrong);
    if (changed) h.setProfile({ ...wrong, account_id: "third-member" });
    h.finish({ ...team, state: "ready" });
    await task;
    assert.equal(h.logic.messageFor(wrong), changed ? "auth_usage_unavailable" : "auth_updated");
  }
});

test("login is single-flight and refreshes the verified provider after login", async () => {
  const h = harness();
  const task = h.logic.start(team);
  assert.equal(h.logic.pending(), true);
  assert.deepEqual(h.calls, [["login", team.key]]);
  await h.logic.start(team);
  h.setProfile({ ...team, state: "ready", can_reauthenticate: false, has_row: true });
  h.finish();
  await task;
  assert.deepEqual(h.stages, ["auth_waiting", "auth_refreshing", "auth_updated"]);
  assert.equal(h.logic.pending(), false);
  assert.deepEqual(h.calls, [["login", team.key], ["refresh", "claude"]]);
});

test("a replaced profile does not receive the old action, refresh or success message", async () => {
  const h = harness();
  const other = { ...team, scope: "personal", email: "other@example.com" };
  h.setProfile(other);
  await h.logic.start(team);
  assert.deepEqual(h.calls, []);
  h.setProfile(team);
  const task = h.logic.start(team);
  h.setProfile(other);
  h.finish();
  await task;
  assert.deepEqual(h.calls, [["login", team.key]]);
  assert.equal(h.logic.messageFor(other), null);
  assert.equal(h.logic.messageFor(team), "auth_profile_changed");
});

test("a different Claude organization or an empty response does not establish updated usage", async () => {
  for (const rows of [null, [], [{ account_id: team.account_id, org_id: "personal", usage: { five_hour: { pct: 20 } } }],
    [{ account_id: "someone-else", org_id: team.org_id, usage: { five_hour: { pct: 20 } } }],
    [{ account_id: team.account_id, org_id: team.org_id, usage: {} }]]) {
    const h = harness();
    h.setResults(rows);
    const task = h.logic.start(team);
    h.finish();
    await task;
    assert.equal(h.logic.messageFor(team), "auth_usage_unavailable");
  }
});

test("a first login can confirm fresh usage before the next profile report assigns IDs", async () => {
  const missing = { ...team, state: "login_required", account_id: null, org_id: null };
  const h = harness(missing);
  const task = h.logic.start(missing);
  h.finish({ ...team, state: "ready" });
  await task;
  assert.equal(h.logic.messageFor(missing), "auth_updated");
});

test("a native reply for a different registered target cannot trigger a refresh", async () => {
  const h = harness();
  const task = h.logic.start(team);
  h.finish({ ...team, scope: "personal" });
  await task;
  assert.deepEqual(h.calls, [["login", team.key]]);
  assert.equal(h.logic.messageFor(team), "auth_profile_changed");
});

test("a Codex team seat only confirms usage from its pinned workspace on first login", async () => {
  const missing = { ...team, provider: "codex", state: "login_required", account_id: null, org_id: null };
  const verified = { ...missing, state: "ready", account_id: "active-workspace", seat_annotation: {
    workspace_id: "pinned-ws", workspace_name: "Design", seat_type: "standard", email: missing.email, verified_at_ms: 1,
  } };
  for (const [id, expected] of [["active-workspace", "auth_usage_unavailable"], ["pinned-ws", "auth_updated"]]) {
    const h = harness(missing);
    h.setResults([{ account_id: id, email: missing.email, plan_type: "business", windows: [{ used_percent: 0 }] }]);
    const task = h.logic.start(missing);
    h.finish(verified);
    await task;
    assert.equal(h.logic.messageFor(missing), expected);
  }
});

test("Codex success requires a fresh reading for that email, workspace and scope", async () => {
  const codex = { ...team, provider: "codex", account_id: "shared-ws", org_id: null };
  const row = { email: team.email.toUpperCase(), account_id: "shared-ws", plan_type: "business", windows: [{ used_percent: 0 }] };
  for (const [patch, expected] of [[{}, "auth_updated"], [{ email: "other@example.com" }, "auth_usage_unavailable"],
    [{ account_id: "another-ws" }, "auth_usage_unavailable"], [{ plan_type: "free" }, "auth_usage_unavailable"]]) {
    const h = harness(codex);
    h.setResults([{ ...row, ...patch }]);
    const task = h.logic.start(codex);
    h.finish();
    await task;
    assert.equal(h.logic.messageFor(codex), expected);
  }
});

test("failed login does not refresh and raw provider errors are not surfaced", async () => {
  for (const [error, expected] of [["auth_wrong_scope", "auth_wrong_scope"], ["unexpected diagnostic", "auth_login_failed"]]) {
    const h = harness();
    const task = h.logic.start(team);
    h.fail(error);
    await task;
    assert.deepEqual(h.calls, [["login", team.key]]);
    assert.equal(h.logic.messageFor(team), expected);
  }
});
