const test = require("node:test");
const assert = require("node:assert/strict");
const guide = require("../ui/model-guide.js");

test("unknown providers, plans, malformed values, and old empty payloads have no invented guide", () => {
  for (const input of [undefined, null, [], {}, { provider: "future" },
    { provider: "agy" }, { provider: "agy", planName: "Google AI Ultra" },
    { provider: "claude", orgType: "claude_future", planName: "Ultra" },
    { provider: "claude", orgType: "claude_team\n", planName: "Ultra" },
    { provider: "claude", orgType: 42, planName: "Ultra" },
    { provider: "claude", planName: "toString" }, { provider: "claude", planName: "Ultra\n" },
  ]) assert.equal(guide.lookup(input), null);
});

test("Claude personal Max and Ultra share included Fable conditions while Pro uses credits", () => {
  for (const orgType of ["claude_max", "claude_ultra"]) {
    const result = guide.lookup({ provider: "claude", orgType, planName: "Ultra" });
    assert.match(result.lines.join("\n"), /Opus 5\.5.*Sonnet 5\.5.*Haiku 5\.5/);
    assert.match(result.lines.join("\n"), /Fable 5\.2: 플랜에 포함/);
    assert.match(result.lines.join("\n"), /공유 주간 한도의 최대 50%/);
    assert.match(result.lines.join("\n"), /2\.1\.321/);
  }
  assert.match(guide.lookup({ provider: "claude", orgType: "claude_pro" }).lines.join("\n"), /기본 사용 한도에 미포함/);
  assert.ok(guide.lookup({ provider: "claude", planName: "Max 5x" }));
});

test("Claude organization type outranks its Ultra or Max usage badge and unknown seats remain conditional", () => {
  const account = { provider: "claude", orgType: "claude_team", planName: "Ultra" };
  const unknown = guide.lookup(account).lines.join("\n");
  assert.match(unknown, /Team Premium.*Standard/);
  assert.match(unknown, /좌석 유형 확인 필요/);
  assert.match(unknown, /조직 관리자의 설정/);
  assert.doesNotMatch(unknown, /Fable 5\.2: 플랜에 포함/);
  assert.match(guide.lookup({ ...account, seatType: "premium" }).lines.join("\n"), /Fable 5\.2: 플랜에 포함/);
  assert.match(guide.lookup({ ...account, seatType: "standard" }).lines.join("\n"), /기본 사용 한도에 미포함/);
});

test("Antigravity Pro lists conditional offerings without inventing account access from its tier name", () => {
  const result = guide.lookup({ provider: "agy", planName: "Google AI Pro" });
  const lines = result.lines.join("\n");
  assert.match(lines, /Gemini 4 Neon.*Argon.*Hydrogen/);
  assert.match(lines, /Claude Sonnet 5\.5.*Opus 5\.5/);
  assert.match(lines, /체험판/);
  assert.match(lines, /Hydrogen·Claude 5\.5: 유료 Pro만/);
  assert.doesNotMatch(result.lines[0], /Carbon/);
  assert.match(lines, /2026-11-30 제거 예정/);
  assert.match(lines, /Antigravity 앱 모델 선택기/);
  assert.match(result.source, /Google Antigravity.*요금제 안내.*2026-11-23/);
  assert.deepEqual(guide.lookup({ provider: " ANTIGRAVITY ", planName: " Google AI Pro " }), result);
});

test("Codex paid plans list documented model examples and minimum CLI versions", () => {
  for (const planType of ["plus", "pro", "prolite", "promax", "business", "team"]) {
    const result = guide.lookup({ provider: "codex", planType });
    assert.match(result.lines.join("\n"), /GPT-6\.2 Astra.*GPT-6\.2 Sol.*GPT-6\.1 Terra/);
    assert.match(result.lines.join("\n"), /0\.166\.0/);
    assert.match(result.lines.join("\n"), /0\.159\.0/);
    assert.match(result.lines.join("\n"), /CLI \/model/);
    assert.match(result.source, /OpenAI.*2026-11-23/);
  }
});

test("Codex pending verified seats get conditional Business guidance without model exclusivity", () => {
  const standard = guide.lookup({ provider: "codex", seatType: "standard" });
  const premium = guide.lookup({ provider: "codex", seatType: "premium" });
  assert.deepEqual(standard, premium);
  assert.match(standard.lines.join("\n"), /좌석과 워크스페이스 권한/);
  assert.equal(guide.lookup({ provider: "codex" }), null);
  assert.equal(guide.lookup({ provider: "codex", planType: "future", seatType: "standard" }), null);
  assert.equal(guide.lookup({ provider: "codex", seatType: "future" }), null);
  assert.equal(guide.lookup({ provider: "codex", planType: "free" }), null);
});

test("all guides have localized provenance and return independent data with no requests", () => {
  for (const locale of ["ko", "en-US", "zh-CN"]) {
    const result = guide.lookup({ provider: "claude", orgType: "claude_team" }, locale);
    assert.equal(typeof result.title, "string");
    assert.ok(result.lines.every((line) => typeof line === "string" && line.length > 0));
    assert.match(result.source, /2026-11-23/);
    assert.match(result.source, /Anthropic/);
  }
  assert.equal(guide.lookup({ provider: "claude", planName: "Ultra" }, "en").title, "Supported models");
  assert.equal(guide.lookup({ provider: "claude", planName: "Ultra" }, "zh").title, "支持模型");
  const first = guide.lookup({ provider: "claude", planName: "Ultra" });
  first.lines.push("changed");
  assert.ok(!guide.lookup({ provider: "claude", planName: "Ultra" }).lines.includes("changed"));
});
