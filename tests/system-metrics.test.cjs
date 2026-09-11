const test = require("node:test");
const assert = require("node:assert/strict");
const metrics = require("../ui/system-metrics.js");

test("unknown readings stay unknown; measured zero is a valid idle reading", () => {
  for (const value of [null, undefined, NaN, Infinity, -1, "0"]) {
    assert.equal(metrics.formatRate(value), "--");
    assert.equal(metrics.formatPercent(value), "--");
  }
  assert.equal(metrics.formatRate(0), "0 B/s");
  assert.equal(metrics.formatPercent(0), "0%");
  assert.equal(metrics.formatPercent(240), "100%");
  assert.equal(metrics.combinedRate({ rx: 0, tx: 0 }, "rx", "tx"), 0);
  assert.equal(metrics.combinedRate({ rx: 100, tx: null }, "rx", "tx"), null);
});

test("throughput uses byte-per-second units, including rounded unit boundaries", () => {
  assert.equal(metrics.formatRate(2_400_000), "2.4 MB/s");
  assert.equal(metrics.formatRate(2_400_000, true), "2.4M/s");
  assert.equal(metrics.formatRate(70_000, true), "70K/s");
  assert.equal(metrics.formatRate(999_999), "1 MB/s");
  assert.equal(metrics.formatBytes(32_000_000_000), "32 GB");
});

test("time slots preserve outages and replace duplicate reports without inventing history", () => {
  const snapshot = {
    sampled_at_ms: 100_000, interval_ms: 2000,
    history: [
      { at_ms: 96_000, cpu_pct: 11 },
      { at_ms: 100_000, cpu_pct: 20 },
      { at_ms: 90_000, cpu_pct: 0 },
      { at_ms: 40_000, cpu_pct: 100 }, // older than the displayed minute
      { at_ms: 102_000, cpu_pct: 100 }, // future samples must not wrap
      { at_ms: null, cpu_pct: 100 },
    ],
    current: { at_ms: 100_000, cpu_pct: 24 },
  };
  const slots = metrics.historySlots(snapshot);
  assert.equal(slots.length, 30);
  assert.equal(slots.at(-1).cpu_pct, 24);
  assert.equal(slots.at(-2), null); // missing 98 s is a gap
  assert.equal(slots.at(-3).cpu_pct, 11);
  assert.equal(slots.at(-6).cpu_pct, 0);
  assert.equal(slots.filter(Boolean).length, 3);
  assert.deepEqual(metrics.historySlots(snapshot), slots);
  assert.equal(metrics.historySlots(undefined).filter(Boolean).length, 0);
});

test("small sampling jitter stays in its time slot while a long pause leaves gaps", () => {
  const slots = metrics.historySlots({
    sampled_at_ms: 60_100, interval_ms: 2000,
    history: [{ at_ms: 58_040, cpu_pct: 12 }, { at_ms: 50_080, cpu_pct: 33 }],
    current: { at_ms: 60_100, cpu_pct: null },
  });
  assert.equal(slots.at(-1).cpu_pct, null);
  assert.equal(slots.at(-2).cpu_pct, 12);
  assert.equal(slots.at(-3), null);
  assert.equal(slots.at(-6).cpu_pct, 33);
});

test("throughput scale accommodates bursts, retains peaks, and releases slowly", () => {
  const floor = 1_000_000;
  assert.equal(metrics.nextScale(undefined, [0, null], floor), floor);
  const burst = metrics.nextScale(floor, [8_000_000], floor);
  assert.ok(burst >= 8_000_000);
  assert.equal(metrics.nextScale(burst, [8_000_000, 0], floor), burst);
  assert.equal(metrics.nextScale(burst, [0], floor, 0), burst);
  const falling = metrics.nextScale(burst, [0], floor, 2000);
  assert.ok(falling < burst && falling > burst * 0.9);
  assert.equal(metrics.nextScale(burst, [0], floor, 600_000), floor);
});

test("browser fixtures match the real optional-GPU report contract", () => {
  const supported = metrics.demoSnapshot(100_000, true);
  const unsupported = metrics.demoSnapshot(100_000, false);
  assert.equal(supported.history.length, 30);
  assert.equal(supported.current.at_ms, supported.sampled_at_ms);
  assert.equal(supported.history[0].at_ms, 42_000);
  assert.equal(supported.gpu.name, "Demo GPU");
  assert.equal(unsupported.gpu, null);
  assert.ok(unsupported.history.every((sample) => sample.gpu_pct === null));
});
