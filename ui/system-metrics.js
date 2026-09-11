// Small, passive system history. No timers or network calls: backend reports
// own sampling and history; repeated renders never manufacture new samples.
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root && root.document) root.QhudSystemMetrics = api;
})(typeof window === "undefined" ? null : window, function () {
  "use strict";

  const SLOTS = 30;
  const number = (v) => typeof v === "number" && Number.isFinite(v) && v >= 0;
  const percent = (v) => number(v) ? Math.min(100, v) : null;
  const formatPercent = (v) => percent(v) == null ? "--" : `${Math.round(percent(v))}%`;

  function formatBytes(value, compact = false) {
    if (!number(value)) return "--";
    const units = compact ? ["B", "K", "M", "G", "T", "P"] : ["B", "KB", "MB", "GB", "TB", "PB"];
    let n = value;
    let unit = 0;
    while (n >= 1000 && unit < units.length - 1) { n /= 1000; unit++; }
    let rounded = Number(n.toFixed(n > 0 && n < 10 && unit > 0 ? 1 : 0));
    // Keep rounded values on the proper side of a unit boundary.
    if (rounded >= 1000 && unit < units.length - 1) { rounded /= 1000; unit++; }
    return `${rounded}${compact ? "" : " "}${units[unit]}`;
  }

  function formatRate(value, compact = false) {
    return number(value) ? `${formatBytes(value, compact)}/s` : "--";
  }

  function combinedRate(sample, first, second) {
    return sample && number(sample[first]) && number(sample[second])
      ? sample[first] + sample[second] : null;
  }

  function historySlots(snapshot, slots = SLOTS) {
    const result = Array(slots).fill(null);
    const end = snapshot?.sampled_at_ms;
    const interval = number(snapshot?.interval_ms) && snapshot.interval_ms > 0
      ? snapshot.interval_ms : 2000;
    if (!number(end)) return result;
    const samples = [...(Array.isArray(snapshot.history) ? snapshot.history : [])];
    if (snapshot.current) samples.push(snapshot.current);
    for (const sample of samples) {
      if (!sample || !number(sample.at_ms) || sample.at_ms > end) continue;
      const age = Math.round((end - sample.at_ms) / interval);
      if (age >= slots) continue;
      const index = slots - 1 - age;
      // A repeated report replaces its slot; it never advances the graph.
      if (!result[index] || sample.at_ms >= result[index].at_ms) result[index] = sample;
    }
    return result;
  }

  function nextScale(previous, values, floor, elapsedMs = 2000) {
    const peak = values.reduce((max, v) => number(v) ? Math.max(max, v) : max, 0);
    const wanted = Math.max(floor, peak * 1.2);
    if (!number(previous) || previous < wanted) return wanted;
    // Expand immediately, then release slowly after the 60-second peak expires.
    // No scale movement on selection, resize, or a duplicate report.
    const decay = Math.pow(0.5, Math.max(0, elapsedMs) / 30000);
    return Math.max(wanted, previous * decay);
  }

  const definitions = [
    { key: "cpu", label: "CPU", field: "cpu_pct" },
    { key: "memory", label: "MEM", field: "memory_pct" },
    { key: "gpu", label: "GPU", field: "gpu_pct" },
    { key: "disk", label: "DISK", first: "disk_read_bps", second: "disk_write_bps", prefixes: ["R", "W"], floor: 1e6 },
    { key: "network", label: "NET", first: "network_rx_bps", second: "network_tx_bps", prefixes: ["↓", "↑"], floor: 1e5 },
  ];
  let snapshot = null;
  let stale = false;
  let selected = null;
  let scaleAt = null;
  let scales = {};
  let nodes = null;

  function create(tag, className, text) {
    const node = document.createElement(tag);
    node.className = className;
    if (text != null) node.textContent = text;
    return node;
  }

  function ensureNodes() {
    if (nodes) return nodes;
    const container = document.getElementById("systemMetrics");
    const grid = document.getElementById("systemGrid");
    const detail = document.getElementById("systemDetail");
    if (!container || !grid || !detail) return null;
    nodes = { container, grid, detail, cells: {} };
    for (const def of definitions) {
      const button = create("button", `system-cell${def.field ? "" : " system-throughput"}`);
      button.type = "button";
      button.dataset.metric = def.key;
      button.setAttribute("aria-controls", "systemDetail");
      button.setAttribute("aria-expanded", "false");
      const label = create("span", "system-label", def.label);
      const value = create("span", "system-value");
      const chart = create("span", "system-chart");
      chart.setAttribute("aria-hidden", "true");
      const bars = Array.from({ length: SLOTS }, () => create("span", "system-bar"));
      chart.append(...bars);
      button.append(label, value, chart);
      button.addEventListener("click", () => {
        selected = selected === def.key ? null : def.key;
        paint();
      });
      // The strip is a control surface, independent of the footer's drag region.
      button.addEventListener("pointerdown", (event) => event.stopPropagation());
      grid.append(button);
      nodes.cells[def.key] = { button, value, chart, bars };
    }
    return nodes;
  }

  const capacity = (used, total) => number(used) && number(total) && total > 0
    ? `${formatBytes(used)} / ${formatBytes(total)}` : "--";

  function description(def, current) {
    const history = "Last 60 s · 2 s samples";
    const unavailable = stale ? " · samples stalled" : "";
    if (def.key === "cpu") {
      const cores = number(snapshot.cpu_count) && snapshot.cpu_count > 0 ? ` · ${snapshot.cpu_count} logical CPUs` : "";
      return `CPU ${formatPercent(current.cpu_pct)}${cores}${unavailable}\n${history} · fixed 0–100%`;
    }
    if (def.key === "memory") {
      return `Memory ${formatPercent(current.memory_pct)} · ${capacity(current.memory_used_bytes, current.memory_total_bytes)}${unavailable}\n${history} · fixed 0–100%`;
    }
    if (def.key === "gpu") {
      const name = snapshot.gpu?.name || "GPU";
      const memory = !stale && number(current.gpu_pct)
        ? ` · VRAM ${capacity(snapshot.gpu?.memory_used_bytes, snapshot.gpu?.memory_total_bytes)}` : "";
      return `${name} · ${formatPercent(current.gpu_pct)}${memory}${unavailable}\n${history} · fixed 0–100%`;
    }
    const scale = formatRate(scales[def.key]);
    if (def.key === "disk") {
      return `Read ${formatRate(current.disk_read_bps)} · Write ${formatRate(current.disk_write_bps)}${unavailable}\nStorage ${capacity(current.disk_used_bytes, current.disk_total_bytes)}\n${history} · read + write · graph ceiling ${scale}`;
    }
    return `Receive ${formatRate(current.network_rx_bps)} · Send ${formatRate(current.network_tx_bps)}${unavailable}\nNon-loopback interfaces · ${history}\nReceive + send · graph ceiling ${scale}`;
  }

  function paint() {
    const ui = ensureNodes();
    if (!ui) return;
    ui.container.hidden = !snapshot;
    if (!snapshot) return;
    ui.container.classList.toggle("is-stale", stale);
    const current = stale ? {} : snapshot.current || {};
    const slots = historySlots(snapshot);
    const gpuAvailable = !!snapshot.gpu;
    ui.grid.classList.toggle("has-gpu", gpuAvailable);
    if (selected === "gpu" && !gpuAvailable) selected = null;
    for (const def of definitions) {
      const cell = ui.cells[def.key];
      cell.button.hidden = def.key === "gpu" && !gpuAvailable;
      if (cell.button.hidden) continue;
      if (def.field) {
        cell.value.textContent = formatPercent(current[def.field]);
      } else {
        if (!cell.value.children.length) {
          cell.value.append(create("span", "system-rate"), create("span", "system-rate"));
        }
        [def.first, def.second].forEach((field, index) => {
          cell.value.children[index].textContent = `${def.prefixes[index]} ${formatRate(current[field], true)}`;
        });
      }
      const ceiling = def.field ? 100 : scales[def.key];
      slots.forEach((sample, index) => {
        const value = def.field ? percent(sample?.[def.field]) : combinedRate(sample, def.first, def.second);
        const bar = cell.bars[index];
        bar.classList.toggle("is-gap", value == null);
        bar.classList.toggle("is-zero", value === 0);
        bar.style.height = value == null ? "0" : `${Math.min(100, Math.max(3, value / ceiling * 100))}%`;
      });
      const detail = description(def, current);
      cell.button.title = `${detail}\nClick for details`;
      cell.button.setAttribute("aria-label", detail.replaceAll("\n", ". "));
      cell.button.setAttribute("aria-expanded", String(selected === def.key));
      if (selected === def.key) ui.detail.textContent = detail;
    }
    ui.detail.hidden = !selected;
  }

  function update(data) {
    snapshot = data && typeof data === "object" ? data : null;
    if (snapshot) {
      const slots = historySlots(snapshot);
      const elapsed = scaleAt == null ? 0 : Math.max(0, snapshot.sampled_at_ms - scaleAt);
      for (const def of definitions.filter((item) => !item.field)) {
        scales[def.key] = nextScale(scales[def.key], slots.map((sample) => combinedRate(sample, def.first, def.second)), def.floor, elapsed);
      }
      scaleAt = snapshot.sampled_at_ms;
    } else {
      scales = {};
      scaleAt = null;
    }
    paint();
  }

  function setStale(value) {
    if (stale === !!value) return;
    stale = !!value;
    paint();
  }

  // Deterministic, credential-free browser preview. This is never used by Tauri.
  function demoSnapshot(now = Date.now(), withGpu = true) {
    const history = Array.from({ length: SLOTS }, (_, index) => {
      const wave = Math.sin(index * 0.61);
      return {
        at_ms: now - (SLOTS - 1 - index) * 2000,
        cpu_pct: Math.round(20 + 12 * wave + (index > 18 && index < 24 ? 31 : 0)),
        memory_pct: 46 + Math.round(index / 8),
        memory_used_bytes: 15.6e9, memory_total_bytes: 32e9,
        gpu_pct: withGpu ? Math.round(9 + 7 * Math.cos(index * 0.42)) : null,
        disk_read_bps: Math.max(0, 2.4e6 + 2e6 * wave),
        disk_write_bps: index % 9 < 3 ? 8e5 : 9e4,
        disk_used_bytes: 380e9, disk_total_bytes: 1e12,
        network_rx_bps: Math.max(0, 6e5 + 5e5 * Math.cos(index * 0.47)),
        network_tx_bps: 7e4 + index * 2400,
      };
    });
    return {
      sampled_at_ms: now, interval_ms: 2000, cpu_count: 16,
      current: history.at(-1), history,
      gpu: withGpu ? { name: "Demo GPU", memory_used_bytes: 2.1e9, memory_total_bytes: 8e9 } : null,
    };
  }

  return { update, setStale, demoSnapshot, formatBytes, formatRate, formatPercent, combinedRate, historySlots, nextScale };
});
