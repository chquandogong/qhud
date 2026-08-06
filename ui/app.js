// qhud widget frontend — renders the poll payload emitted by the Rust
// backend ("qhud://report"). No framework, no bundler: the DOM is
// patched in place so 2s refreshes never re-trigger CSS animations.
(() => {
  "use strict";

  const tilesEl = document.getElementById("tiles");
  const summaryEl = document.getElementById("summary");
  const metaEl = document.getElementById("meta");
  const srcBadge = document.getElementById("srcBadge");
  const grip = document.getElementById("grip");

  // localStorage can be denied outright in WebKitGTK for the tauri
  // custom-protocol origin (writes throw SecurityError) — never let
  // persistence break interaction.
  const store = {
    get(k) {
      try {
        return localStorage.getItem(k);
      } catch {
        return null;
      }
    },
    set(k, v) {
      try {
        if (v == null) localStorage.removeItem(k);
        else localStorage.setItem(k, v);
      } catch {}
    },
  };

  const state = {
    payload: null,
    receivedAt: 0, // Date.now() when the payload arrived
    selected: store.get("qhud.selected"),
    nodes: new Map(), // pane_id -> tile element
  };

  // ---- formatting -------------------------------------------------

  const sev = (pct) =>
    pct < 60 ? "good" : pct < 75 ? "concern" : pct < 85 ? "warn" : "crit";

  function fmtTokens(n) {
    if (n == null) return null;
    if (n >= 1e6)
      return (
        (n / 1e6)
          .toFixed(2)
          .replace(/\.?0+$/, (m) => (m === ".00" ? ".00" : "")) + "M"
      );
    if (n >= 1e3) return Math.round(n / 1e3) + "K";
    return String(n);
  }

  function fmtReset(unix) {
    const diff = unix * 1000 - Date.now();
    if (diff <= 0) return "now";
    const mins = Math.ceil(diff / 60000);
    if (mins < 60) return `${mins}m`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) {
      const mm = mins % 60;
      return mm ? `${hours}h${String(mm).padStart(2, "0")}m` : `${hours}h`;
    }
    const days = Math.floor(hours / 24);
    const hh = hours % 24;
    return hh ? `${days}d ${hh}h` : `${days}d`;
  }

  function fmtElapsed(baseSecs) {
    const drift = Math.max(
      0,
      Math.floor((Date.now() - state.receivedAt) / 1000),
    );
    const s = baseSecs + drift;
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  }

  function clock(ms) {
    const d = new Date(ms);
    const p = (n) => String(n).padStart(2, "0");
    return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
  }

  const el = (tag, cls, text) => {
    const n = document.createElement(tag);
    if (cls) n.className = cls;
    if (text != null) n.textContent = text;
    return n;
  };

  // ---- tile construction / patching -------------------------------

  function buildTile(pane) {
    const tile = el("section", "tile compact");
    tile.dataset.paneId = pane.pane_id;

    const head = el("div", "head");
    head.append(
      el("span", "pane"),
      el("span", "conflict-mark", "⚠"),
      el("span", "ver"),
      el("span", "pill"),
    );
    tile.append(
      head,
      el("div", "cfg"),
      el("div", "gauges"),
      el("div", "conflict"),
    );

    // pointerdown, not click: click synthesis needs a clean
    // down+up pair, while pointerdown delivery is beacon-proven on
    // this webview. Render first — persistence must never gate UI.
    tile.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      state.selected = state.selected === pane.pane_id ? null : pane.pane_id;
      render();
      store.set("qhud.selected", state.selected);
      if (window.__qhudBeacon)
        window.__qhudBeacon(
          `sel:${state.selected || "none"}:${tile.classList.contains("selected") ? "R" : "-"}`,
        );
    });
    return tile;
  }

  function gaugeRow(key) {
    const row = el("div", "gauge");
    row.dataset.key = key;
    const track = el("span", "g-track");
    track.append(el("span", "g-fill"));
    const val = el("span", "g-val");
    val.append(document.createTextNode(""), el("i", null, "%"));
    row.append(
      el("span", "g-label", key.toUpperCase()),
      track,
      val,
      el("span", "g-reset"),
    );
    return row;
  }

  function patchGauges(tile, pane) {
    const wrap = tile.querySelector(".gauges");
    const defs = [
      ["ctx", pane.gauges.ctx],
      ["5h", pane.gauges.h5],
      ["7d", pane.gauges.d7],
    ];
    for (const [key, g] of defs) {
      let row = wrap.querySelector(`[data-key="${key}"]`);
      if (!g) {
        if (row) row.remove();
        continue;
      }
      if (!row) {
        row = gaugeRow(key);
        wrap.append(row);
      }
      row.dataset.sev = sev(g.pct);
      row.title = g.source ? `source: ${g.source}` : "";
      row.querySelector(".g-fill").style.width = g.pct + "%";
      row.querySelector(".g-val").firstChild.nodeValue = g.pct;
      const reset = row.querySelector(".g-reset");
      if (g.reset_unix) {
        reset.dataset.resetUnix = g.reset_unix;
        reset.classList.toggle(
          "soon",
          g.reset_unix * 1000 - Date.now() < 60 * 60000,
        );
        reset.innerHTML = "";
        reset.append("resets ", el("b", null, fmtReset(g.reset_unix)));
      } else {
        delete reset.dataset.resetUnix;
        reset.classList.remove("soon");
        const of = fmtTokens(g.of_tokens);
        reset.textContent = of ? `of ${of}` : "";
      }
    }
  }

  function patchCfg(tile, pane) {
    const cfg = tile.querySelector(".cfg");
    cfg.innerHTML = "";
    const items = [];
    const push = (k, v, cls) => v && items.push([k, v, cls]);
    push("model", pane.model);
    push("effort", pane.effort);
    for (const f of pane.flags || []) items.push([null, f, "flag"]);
    push("branch", pane.branch);
    push("cwd", pane.cwd);
    push("mem", pane.mem);
    if (pane.cost_usd != null) push("cost", `$${pane.cost_usd.toFixed(2)}`);
    for (const [k, v, cls] of items) {
      const item = el("span", "item" + (cls ? " " + cls : ""));
      if (k) item.append(el("span", "k", k));
      item.append(el("span", "v", v));
      cfg.append(item);
    }
    cfg.hidden = items.length === 0;
  }

  function patchConflict(tile, pane) {
    const box = tile.querySelector(".conflict");
    box.innerHTML = "";
    const c = (pane.conflicts || [])[0];
    if (!c) {
      box.hidden = true;
      return;
    }
    box.hidden = false;
    box.append(el("span", "mark", "⚠ CONFLICT"));
    if (c.paths && c.paths.length)
      box.append(el("span", "file", c.paths.join(", ")));
    else box.append(el("span", null, c.reason));
    if (c.peers && c.peers.length) {
      box.append("· with ");
      box.append(el("span", "peer", c.peers.join(", ")));
    }
  }

  function patchTile(tile, pane) {
    const expanded = state.selected === pane.pane_id;
    tile.classList.toggle("selected", expanded);
    tile.classList.toggle("compact", !expanded);

    tile.querySelector(".pane").textContent = pane.label;

    const mark = tile.querySelector(".conflict-mark");
    mark.hidden = !(pane.conflicts && pane.conflicts.length);

    const ver = tile.querySelector(".ver");
    ver.innerHTML = "";
    if (pane.cli_version) {
      ver.append(`v${pane.cli_version}`);
      if (pane.update_hint) ver.append(el("i", "upd", `↑ ${pane.update_hint}`));
    }

    const pill = tile.querySelector(".pill");
    pill.className = `pill ${pane.status}`;
    pill.innerHTML = "";
    pill.append(el("i", "led"), pane.status_label);
    if (pane.elapsed_secs != null) {
      const sp = el("b", "elapsed", fmtElapsed(pane.elapsed_secs));
      sp.dataset.baseSecs = pane.elapsed_secs;
      pill.append(" ", sp);
    }

    // Detail sections only render on the expanded tile (mockup: the
    // selected pane shows config + conflict, others stay compact).
    if (expanded) {
      patchCfg(tile, pane);
      patchConflict(tile, pane);
    } else {
      const cfg = tile.querySelector(".cfg");
      cfg.hidden = true;
      const conflict = tile.querySelector(".conflict");
      conflict.hidden = true;
    }
    patchGauges(tile, pane);
  }

  // ---- top-level render -------------------------------------------

  function render() {
    const p = state.payload;
    if (!p) return;

    // summary line
    summaryEl.innerHTML = "";
    summaryEl.append(`${p.summary.panes} panes · `);
    const conf = el(
      "span",
      p.summary.conflicts ? "s-conf" : null,
      `${p.summary.conflicts} conflict`,
    );
    summaryEl.append(conf);
    if (p.summary.max_5h_pct != null) {
      summaryEl.append(" · 5H ");
      summaryEl.append(el("b", null, `${p.summary.max_5h_pct}%`));
    }

    srcBadge.hidden = p.source !== "demo";

    // tiles (keyed patch, order = payload order)
    const alive = new Set();
    let prev = null;
    for (const pane of p.panes) {
      alive.add(pane.pane_id);
      let tile = state.nodes.get(pane.pane_id);
      if (!tile) {
        tile = buildTile(pane);
        state.nodes.set(pane.pane_id, tile);
      }
      patchTile(tile, pane);
      // keep DOM order in sync without re-appending settled nodes
      if (
        prev
          ? tile.previousElementSibling !== prev
          : tilesEl.firstElementChild !== tile
      ) {
        prev ? prev.after(tile) : tilesEl.prepend(tile);
      }
      prev = tile;
    }
    for (const [id, node] of state.nodes) {
      if (!alive.has(id)) {
        node.remove();
        state.nodes.delete(id);
      }
    }

    if (p.panes.length === 0) {
      if (!tilesEl.querySelector(".empty")) {
        const empty = el("div", "empty");
        empty.append(
          "no AI CLI panes found — start ",
          el("code", null, "claude / codex / gemini"),
          " inside tmux",
        );
        tilesEl.append(empty);
      }
    } else {
      const empty = tilesEl.querySelector(".empty");
      if (empty) empty.remove();
    }

    const src = p.backend ? `${p.source}·${p.backend}` : p.source;
    metaEl.textContent = `${src} · poll ${p.poll_secs}s · updated ${clock(p.generated_at_ms)}`;
  }

  // 1s ticker: countdowns and elapsed badges advance between polls.
  setInterval(() => {
    if (!state.payload) return;
    // Stale watchdog (cross-validation CV-2): if the backend stops
    // emitting, say so instead of freezing plausible-looking numbers.
    metaEl.classList.toggle(
      "stale",
      state.receivedAt > 0 && Date.now() - state.receivedAt > 8000,
    );
    for (const reset of tilesEl.querySelectorAll(".g-reset[data-reset-unix]")) {
      const b = reset.querySelector("b");
      if (b) b.textContent = fmtReset(Number(reset.dataset.resetUnix));
    }
    for (const sp of tilesEl.querySelectorAll(".elapsed[data-base-secs]")) {
      sp.textContent = fmtElapsed(Number(sp.dataset.baseSecs));
    }
  }, 1000);

  // ---- wiring ------------------------------------------------------

  document.addEventListener("contextmenu", (e) => e.preventDefault());

  const tauri = window.__TAURI__;
  if (tauri) {
    const win = tauri.window.getCurrentWindow();
    const PhysicalPos =
      (tauri.dpi && tauri.dpi.PhysicalPosition) ||
      tauri.window.PhysicalPosition;
    const PhysicalSize =
      (tauri.dpi && tauri.dpi.PhysicalSize) || tauri.window.PhysicalSize;

    // Stderr breadcrumbs (permanent, invisible): interaction events go
    // to the Rust side so real-input behavior is verifiable from logs.
    const crumb = (s) => {
      try {
        tauri.core.invoke("ui_event", { event: s }).catch(() => {});
      } catch {}
    };
    window.__qhudBeacon = crumb;

    tauri.event.listen("qhud://report", ({ payload }) => {
      state.payload = payload;
      state.receivedAt = Date.now();
      render();
    });

    // Manual move/resize: compositor-side interactive move/resize
    // (startDragging / drag regions / tao edge grabs) is unreliable
    // for a keep-below XWayland window on GNOME, and WebKitGTK's
    // event.screenX/Y goes stale while the window itself moves. So
    // geometry is driven by an rAF loop over Tauri's *global*
    // cursorPosition(), which is immune to both problems: pointer
    // events only arm/disarm the loop.
    const cursorPosition = tauri.window.cursorPosition;
    const manual = { mode: null, grabX: 0, grabY: 0, baseW: 0, baseH: 0 };

    async function beginManual(mode, e) {
      try {
        e.currentTarget.setPointerCapture(e.pointerId);
      } catch {}
      // Deliberately no outerPosition()/outerSize(): tao mis-reports
      // both by a phantom frame height (~top-bar sized) for this
      // undecorated X11 window. The DOM already knows the truth:
      // clientX/Y is the pointer's offset inside the window, and
      // innerWidth/Height is the real window size (frameless ⇒
      // client area == window).
      const scale = window.devicePixelRatio || 1;
      if (mode === "move") {
        manual.grabX = Math.round(e.clientX * scale);
        manual.grabY = Math.round(e.clientY * scale);
      } else {
        const cur = await cursorPosition();
        manual.grabX = cur.x;
        manual.grabY = cur.y;
        manual.baseW = Math.round(window.innerWidth * scale);
        manual.baseH = Math.round(window.innerHeight * scale);
      }
      manual.mode = mode;
      requestAnimationFrame(step);
    }

    async function step() {
      if (!manual.mode) return;
      try {
        const cur = await cursorPosition();
        if (manual.mode === "move") {
          await win.setPosition(
            new PhysicalPos(
              Math.round(cur.x - manual.grabX),
              Math.round(cur.y - manual.grabY),
            ),
          );
        } else if (manual.mode === "resize") {
          await win.setSize(
            new PhysicalSize(
              Math.max(320, Math.round(manual.baseW + cur.x - manual.grabX)),
              Math.max(200, Math.round(manual.baseH + cur.y - manual.grabY)),
            ),
          );
        }
      } catch (err) {
        console.error("qhud manual drag:", err);
      }
      if (manual.mode) requestAnimationFrame(step);
    }

    const endManual = () => {
      manual.mode = null;
    };
    document.addEventListener("pointerup", endManual);
    document.addEventListener("pointercancel", endManual);
    window.addEventListener("blur", endManual);

    for (const bar of document.querySelectorAll(".topbar, .foot")) {
      bar.addEventListener("pointerdown", (e) => {
        if (e.target.closest(".grip")) return;
        e.preventDefault();
        beginManual("move", e);
      });
    }
    grip.addEventListener("pointerdown", (e) => {
      e.preventDefault();
      beginManual("resize", e);
    });
  } else {
    // Browser preview (no Tauri runtime): render a static sample so
    // ui/index.html can be opened directly during development.
    grip.style.display = "none";
    const now = Math.floor(Date.now() / 1000);
    state.payload = {
      schema: 1,
      source: "demo",
      generated_at_ms: Date.now(),
      poll_secs: 2,
      summary: { panes: 3, conflicts: 1, max_5h_pct: 88 },
      panes: [
        {
          pane_id: "%25",
          label: "claude:1:main",
          provider: "claude",
          status: "active",
          status_label: "active",
          elapsed_secs: null,
          cli_version: "2.1.4",
          update_hint: null,
          model: "opus-4.8",
          effort: "max",
          branch: "main",
          cwd: "~/qhud",
          mem: "48 KB",
          cost_usd: null,
          flags: ["⏵⏵ bypass on"],
          gauges: {
            ctx: {
              pct: 64,
              source: "providerofficial",
              reset_unix: null,
              of_tokens: 1000000,
            },
            h5: {
              pct: 88,
              source: "providerofficial",
              reset_unix: now + 47 * 60,
              of_tokens: null,
            },
            d7: {
              pct: 31,
              source: "providerofficial",
              reset_unix: now + 363600,
              of_tokens: null,
            },
          },
          conflicts: [
            {
              reason: "same-file edits",
              severity: "warning",
              paths: ["src/ui/panels/mod.rs"],
              peers: ["codex:1:review"],
            },
          ],
        },
        {
          pane_id: "%27",
          label: "codex:1:review",
          provider: "codex",
          status: "stale",
          status_label: "idle stale",
          elapsed_secs: 42,
          cli_version: "0.142",
          update_hint: "0.143",
          model: null,
          effort: null,
          branch: null,
          cwd: "~/qhud",
          mem: null,
          cost_usd: null,
          flags: [],
          gauges: {
            ctx: {
              pct: 36,
              source: "providerofficial",
              reset_unix: null,
              of_tokens: 258000,
            },
            h5: {
              pct: 61,
              source: "providerofficial",
              reset_unix: now + 3900,
              of_tokens: null,
            },
            d7: {
              pct: 44,
              source: "providerofficial",
              reset_unix: now + 432000,
              of_tokens: null,
            },
          },
          conflicts: [],
        },
        {
          pane_id: "%28",
          label: "agy:1:research",
          provider: "agy",
          status: "stale",
          status_label: "idle stale",
          elapsed_secs: 35,
          cli_version: "1.0.14",
          update_hint: null,
          model: null,
          effort: null,
          branch: null,
          cwd: "~/research",
          mem: null,
          cost_usd: null,
          flags: [],
          gauges: {
            ctx: {
              pct: 41,
              source: "providerofficial",
              reset_unix: null,
              of_tokens: 1050000,
            },
            h5: {
              pct: 8,
              source: "providerofficial",
              reset_unix: now + 13200,
              of_tokens: null,
            },
            d7: {
              pct: 3,
              source: "providerofficial",
              reset_unix: now + 518400,
              of_tokens: null,
            },
          },
          conflicts: [],
        },
      ],
    };
    state.receivedAt = Date.now();
    render();
  }
})();
