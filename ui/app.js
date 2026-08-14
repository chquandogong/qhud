// qhud widget frontend — renders the poll payload emitted by the Rust
// backend ("qhud://report"). No framework, no bundler: the DOM is
// patched in place so 2s refreshes never re-trigger CSS animations.
(() => {
  "use strict";

  const tilesEl = document.getElementById("tiles");
  const summaryEl = document.getElementById("summary");
  const quotasEl = document.getElementById("quotas");
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

  // Coarse age for stored snapshots — the answer to "how stale is this?"
  function fmtAgeMs(ms) {
    const min = Math.round((Date.now() - ms) / 60000);
    if (min < 90) return `${Math.max(min, 0)}m`;
    const hours = Math.round(min / 60);
    if (hours < 48) return `${hours}h`;
    return `${Math.round(hours / 24)}d`;
  }

  // Minor-unit money (cents when exponent 2) → display string. Money
  // stays integer all the way from the wire; division happens only here.
  function fmtMinor(minor, exponent, currency) {
    const exp = exponent ?? 2;
    const amount = (minor / 10 ** exp).toFixed(exp);
    return currency === "USD" || currency == null
      ? `$${amount}`
      : `${amount} ${currency}`;
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
      el("span", "sess"),
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
    // Tiles show pane-scoped facts only; 5h/7d are account facts and
    // live in the provider quota strip (D-011).
    const defs = [["ctx", pane.gauges.ctx]];
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
    tile.querySelector(".sess").textContent = pane.session
      ? "@" + pane.session
      : "";

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

  // ---- provider quota strip (account-scoped facts, D-011) ---------

  function buildQuotaRow(provider) {
    const row = el("div", "q-row");
    row.dataset.provider = provider;
    row.append(
      el("span", "q-prov", provider),
      el("span", "q-acct"),
      el("span", "q-plan"),
      el("span", "q-extra"),
      el("span", "q-age"),
    );
    if (provider === "claude") {
      const btn = el("button", "q-refresh", "⟳");
      btn.title = "fetch live usage, including per-model windows";
      row.append(btn);
    }
    if (provider === "agy") {
      const btn = el("button", "q-refresh", "⟳");
      btn.title = "read quota from the running agy (loopback, no token)";
      row.append(btn);
    }
    if (provider === "codex") {
      // Fetching used to be the row CLICK, which collided with rows
      // becoming selectable — network stays behind an explicit ⟳.
      const btn = el("button", "q-refresh", "⟳");
      btn.title = "fetch workspace usage (network)";
      row.append(btn);
    }
    for (const win of ["5h", "7d"])
      row.append(buildChip(win, win.toUpperCase()));
    // Selecting the row expands this inline (filled by renderQuotas with
    // the same facts the hover tooltip carries).
    row.append(el("div", "q-detail"));
    return row;
  }

  function buildChip(win, label) {
    const chip = el("span", "q-chip");
    chip.dataset.win = win;
    const track = el("span", "q-track");
    track.append(el("span", "q-fill"));
    const val = el("span", "q-val");
    val.append(document.createTextNode(""), el("i", null, "%"));
    chip.append(
      el("span", "q-label", label),
      track,
      val,
      el("span", "q-reset"),
    );
    return chip;
  }

  function patchQuotaChip(chip, g) {
    if (!g) {
      chip.hidden = true;
      return;
    }
    chip.hidden = false;
    chip.dataset.sev = sev(g.pct);
    chip.querySelector(".q-fill").style.width = g.pct + "%";
    chip.querySelector(".q-val").firstChild.nodeValue = g.pct;
    const reset = chip.querySelector(".q-reset");
    if (g.reset_unix) {
      reset.dataset.resetUnix = g.reset_unix;
      reset.classList.toggle(
        "soon",
        g.reset_unix * 1000 - Date.now() < 60 * 60000,
      );
      reset.textContent = fmtReset(g.reset_unix);
    } else {
      delete reset.dataset.resetUnix;
      reset.classList.remove("soon");
      reset.textContent = "";
    }
  }

  // Codex workspaces (personal / business) live behind ONE login and share
  // its token, so they cost no extra auth — but fetching them leaves the
  // machine, so it happens on an explicit click and never on the timer.
  const codexFetch = { state: "idle", rows: [], error: null, at: 0 };
  // Explicitly refreshed Claude usage. Only this can produce per-model windows
  // (the statusline has none, and nothing qhud can run refreshes the on-disk
  // cache), so it is a click, never a poll.
  const claudeFetch = { state: "idle", data: null, error: null, at: 0 };
  // agy usage via the CLI's own loopback RPC (no token, machine-local) —
  // only answers while agy runs, so it too is a click, never a poll.
  const agyFetch = { state: "idle", data: null, error: null, at: 0 };

  async function refreshAgyUsage() {
    if (agyFetch.state === "loading") return;
    agyFetch.state = "loading";
    agyFetch.error = null;
    render();
    try {
      const inv = window.__TAURI__?.core?.invoke;
      if (!inv) throw new Error("not running under Tauri");
      agyFetch.data = await inv("fetch_agy_usage");
      agyFetch.at = Date.now();
      agyFetch.state = "done";
    } catch (e) {
      agyFetch.state = "error";
      agyFetch.error = String(e);
    }
    render();
  }

  async function refreshClaudeUsage() {
    if (claudeFetch.state === "loading") return;
    claudeFetch.state = "loading";
    claudeFetch.error = null;
    render();
    try {
      const inv = window.__TAURI__?.core?.invoke;
      if (!inv) throw new Error("not running under Tauri");
      claudeFetch.data = await inv("fetch_claude_usage");
      claudeFetch.at = Date.now();
      claudeFetch.state = "done";
    } catch (e) {
      claudeFetch.state = "error";
      claudeFetch.error = String(e);
    }
    render();
    // The command succeeding does not prove the per-model gauges rendered —
    // that distinction is exactly what let a wrong Fable value ship before.
    if (claudeFetch.state === "done") {
      const rows = Array.isArray(claudeFetch.data) ? claudeFetch.data : [];
      const scopedTotal = rows.reduce(
        (n, r) => n + (r.usage?.scoped?.length || 0),
        0,
      );
      window.__TAURI__?.core?.invoke("ui_event", {
        event: `claude-refresh: ${rows.length} accounts, ${scopedTotal} scoped fetched, ${
          quotasEl.querySelectorAll('[data-win^="m:"]').length
        } rendered`,
      });
      logLabels("after-claude-refresh");
    }
  }
  // Provider words are long and the strip is ~520px; shorten without
  // inventing meaning.
  const shortPlan = (s) =>
    ({ claude_team: "team", claude_max: "max", claude_pro: "pro" })[s] || s;
  const shortTier = (s) =>
    String(s)
      .replace(/^default_/, "")
      .replace(/^claude_/, "");

  async function fetchCodexWorkspaces() {
    if (codexFetch.state === "loading") return;
    codexFetch.state = "loading";
    codexFetch.error = null;
    renderCodexExtra();
    try {
      const inv = window.__TAURI__?.core?.invoke;
      if (!inv) throw new Error("not running under Tauri");
      codexFetch.rows = await inv("fetch_codex_workspaces");
      codexFetch.at = Date.now();
      codexFetch.state = "done";
    } catch (e) {
      codexFetch.state = "error";
      codexFetch.error = String(e);
    }
    renderCodexExtra();
  }

  // A workspace's windows in the snapshot shape the row-override path
  // expects (the JS twin of the backend's workspace_snapshot).
  function wsToUsage(w) {
    const u = { five_hour: null, seven_day: null, scoped: [] };
    for (const x of w.windows || []) {
      if (!x.scope && x.label === "5h")
        u.five_hour = { pct: x.used_percent, reset_unix: x.reset_unix };
      else if (!x.scope && x.label === "weekly")
        u.seven_day = { pct: x.used_percent, reset_unix: x.reset_unix };
      else
        u.scoped.push({
          kind: `pool_${x.label}`,
          scope: x.scope || "codex",
          pct: x.used_percent,
          reset_unix: x.reset_unix,
        });
    }
    return u;
  }

  // The workspace rows to show: a fetch made this session always wins;
  // before any fetch, the stored rows from qhud's LAST explicit fetch
  // (riding the payload) render instead — dated, never passing for live.
  function codexSource() {
    if (codexFetch.state === "done")
      return { rows: codexFetch.rows, at: codexFetch.at, stored: false };
    const p = state.payload;
    if (p?.codex_workspaces?.length)
      return {
        rows: p.codex_workspaces,
        at: p.codex_fetched_at_ms || 0,
        stored: true,
      };
    return { rows: [], at: 0, stored: false };
  }

  // Rebuilds are gated on a source signature: renderCodexExtra now runs on
  // every 2 s payload (for the stored rows), and unconditionally recreating
  // the rows would retrigger the gauge CSS transitions each tick. Between
  // rebuilds only the age caption moves.
  let codexRowsSig = null;

  // Render each Codex workspace as its OWN row. One login owns several
  // workspaces (personal + business) with separate quota, so collapsing
  // them into a tooltip hid the very thing the fetch is for.
  function renderCodexExtra() {
    const anchor = quotasEl.querySelector('[data-provider="codex"]');
    if (!anchor) return;
    const ageEl = anchor.querySelector(".q-age");
    const clearRows = () => {
      for (const r of [...quotasEl.querySelectorAll("[data-wsid]")]) r.remove();
    };

    if (codexFetch.state === "loading") {
      codexRowsSig = null;
      clearRows();
      ageEl.hidden = false;
      ageEl.textContent = "fetching…";
      return;
    }
    if (codexFetch.state === "error") {
      codexRowsSig = null;
      clearRows();
      ageEl.hidden = false;
      ageEl.textContent = "fetch failed";
      anchor.title = `${codexFetch.error}\n\n⟳ to retry`;
      return;
    }

    const { rows, at, stored } = codexSource();
    // The ACTIVE login's workspace lives merged on the provider row
    // (same pool as the pane statusline — two rows would be the same 7D
    // twice, clock-skewed); ↳ rows are for the others only.
    const visible = rows.filter((w) => !w.active);
    if (visible.length === 0) {
      // Nothing to hang below the row; provenance lives on the row/tooltip.
      const sig = `${stored}:${at}:0`;
      if (sig !== codexRowsSig) {
        codexRowsSig = sig;
        clearRows();
      }
      return;
    }
    const caption = `${visible.length} ws${at ? ` · ⟳ ${fmtAgeMs(at)} ago` : ""}`;
    const sig = `${stored}:${at}:${visible.length}`;
    ageEl.hidden = false;
    ageEl.textContent = caption;
    if (sig === codexRowsSig) return; // same rows; only the caption moved
    codexRowsSig = sig;
    clearRows();
    // The plan belongs to a WORKSPACE, not the login. Leaving it on the
    // account line made three rows for two workspaces, with the parent
    // duplicating the personal one and reading as a third account.

    // The signed-in login's plan, shown inline on the codex row itself.
    // Deliberately NOT lifting w.plan_type onto the parent row: it is a wire
    // enum ('prolite', 'team'), not the name the operator sees. The display
    // name comes from the registry via account.plan.

    let after = anchor;
    for (const w of visible) {
      const row = el("div", `q-row q-ws${stored ? " q-stored" : ""}`);
      row.dataset.wsid = w.account_id;
      const wins = (w.windows || []).filter((x) => x.used_percent != null);
      row.append(
        el("span", "q-prov", "↳"),
        el(
          "span",
          "q-acct",
          // A hex id prefix is not a label a human can act on. Registry name
          // wins, then whatever accounts/check supplied, and the id is only a
          // last resort.
          (state.payload?.workspace_names || {})[w.account_id] ||
            w.name ||
            w.account_id.slice(0, 8),
        ),
        el(
          "span",
          "q-plan",
          (state.payload?.workspace_plans || {})[w.account_id] || "",
        ),
      );
      for (const x of wins) {
        const chip = el("span", "q-chip");
        chip.dataset.sev = sev(x.used_percent);
        const track = el("span", "q-track");
        track.append(el("span", "q-fill"));
        track.querySelector(".q-fill").style.width = x.used_percent + "%";
        const val = el("span", "q-val");
        val.append(document.createTextNode(x.used_percent), el("i", null, "%"));
        const reset = el("span", "q-reset");
        if (x.reset_unix) {
          reset.dataset.resetUnix = x.reset_unix;
          reset.textContent = fmtReset(x.reset_unix);
        }
        // A per-model pool has the same duration as the main one — the
        // NAME is the only thing telling two 7D chips apart. The version
        // prefix is dropped for width ("GPT-5.3-Codex-Spark" →
        // "Codex-Spark"); the full name stays in the chip tooltip.
        let labelText = winLabel(x.label);
        if (x.scope) {
          const short = x.scope.replace(/^GPT-[\d.]+-/, "");
          labelText = `${short} ${labelText}`;
          chip.title = x.scope;
        }
        chip.append(el("span", "q-label", labelText), track, val, reset);
        row.append(chip);
      }
      if (wins.length === 0) row.append(el("span", "q-age", "no window"));
      const meta = [];
      if (stored)
        meta.push(`from qhud's last fetch, ${fmtAgeMs(at)} ago — ⟳ to refresh`);
      if (w.plan_type) meta.push(`plan_type (wire): ${w.plan_type}`);
      if (w.credits_balance) meta.push(`credits ${w.credits_balance}`);
      if (meta.length) row.title = meta.join("\n");
      const detail = el("div", "q-detail");
      detail.textContent = meta.length
        ? meta.join("\n")
        : `workspace ${w.account_id}`;
      row.append(detail);
      after.after(row);
      after = row;
    }
    // Report what was actually appended: the fetch succeeding in Rust does
    // not prove the workspace rows rendered (the pixels are unverifiable from
    // outside the webview, D-010).
    window.__TAURI__?.core?.invoke("ui_event", {
      event: `codex-ws-rows: ${rows.length} ${stored ? "stored" : "fetched"} (${
        rows.length - visible.length
      } merged into the row), ${
        quotasEl.querySelectorAll("[data-wsid]").length
      } rendered`,
    });
    logLabels(stored ? "stored-codex-rows" : "after-codex-fetch");
  }

  // ONE pointerdown handler for the whole strip (pointerdown, not click:
  // a keep-below widget never receives synthesized clicks reliably,
  // D-009). Selecting a row expands its detail inline — hover tooltips
  // barely exist for a window you are rarely pointing at — and network
  // runs ONLY from the ⟳ buttons, never from selecting a row.
  //
  // History (2026-08-12, operator report "selection doesn't work"): the
  // ptr: breadcrumbs showed every click arriving and landing on strip
  // rows — which had no selection behavior at all, and ghost rows had
  // TWO identical listeners, so their toggle ran twice and visibly did
  // nothing. Selecting was never broken on the tiles; it did not exist
  // where the operator was clicking.
  quotasEl.addEventListener("pointerdown", async (e) => {
    if (e.target.classList?.contains("q-refresh")) {
      const provider = e.target.closest(".q-row")?.dataset.provider;
      if (provider === "agy") refreshAgyUsage();
      else if (provider === "codex") fetchCodexWorkspaces();
      else refreshClaudeUsage();
      return;
    }
    if (e.target.classList?.contains("q-forget")) {
      // Explicit dismissal — persisted, so it stays gone across restarts.
      const ghost = e.target.closest(".q-row[data-pkey]");
      if (!ghost) return;
      const [provider, ...rest] = ghost.dataset.pkey.split(":");
      try {
        await window.__TAURI__?.core?.invoke("forget_account", {
          provider,
          key: rest.join(":"),
        });
        ghost.remove();
      } catch (err) {
        ghost.querySelector(".q-age").textContent = "dismiss failed";
        ghost.title = String(err);
      }
      return;
    }
    const row = e.target.closest?.(".q-row");
    if (!row) return;
    row.classList.toggle("q-open");
    window.__qhudBeacon?.(
      `qsel:${
        row.dataset.provider
          ? `${row.dataset.provider}/${row.dataset.acct || "?"}`
          : row.dataset.pkey || row.dataset.wsid || "?"
      }:${row.classList.contains("q-open") ? "open" : "closed"}`,
    );
  });

  // One gesture, every provider. Each fetch keeps its own state and
  // failure surface, so one provider's 401 never hides another's numbers.
  function refreshAll() {
    refreshClaudeUsage();
    fetchCodexWorkspaces();
    refreshAgyUsage();
  }

  const refreshAllBtn = document.getElementById("refreshAll");
  refreshAllBtn?.addEventListener("pointerdown", (e) => {
    // pointerdown, not click (D-009); stop the topbar drag from starting.
    e.stopPropagation();
    e.preventDefault();
    refreshAll();
  });

  // The topbar button mirrors the union of the per-provider fetch states,
  // so "something is fetching / something failed" is visible even when
  // the strip is scrolled or the failing row is collapsed.
  function patchRefreshAll() {
    if (!refreshAllBtn) return;
    const busy =
      claudeFetch.state === "loading" ||
      codexFetch.state === "loading" ||
      agyFetch.state === "loading";
    refreshAllBtn.textContent = busy ? "…" : "⟳";
    const errs = [claudeFetch.error, codexFetch.error, agyFetch.error].filter(
      Boolean,
    );
    refreshAllBtn.classList.toggle("err", !busy && errs.length > 0);
    refreshAllBtn.title = errs.length
      ? errs.join("\n")
      : "refresh usage for every provider (network)";
  }

  // Non-pointer trigger for the Codex fetch, relayed from `qhud --fetch-codex`
  // (a keep-below widget cannot receive synthesized clicks, D-010).
  window.__TAURI__?.event
    ?.listen("qhud://fetch-codex", () => fetchCodexWorkspaces())
    .catch(() => {});
  window.__TAURI__?.event
    ?.listen("qhud://refresh-claude", () => refreshClaudeUsage())
    .catch(() => {});
  window.__TAURI__?.event
    ?.listen("qhud://refresh-all", () => refreshAll())
    .catch(() => {});

  // Accounts that have connected before but have no live credential now.
  // Shown dimmed and numberless: their quota is still ticking, so hiding
  // them would be a lie of omission, but reading it needs a re-auth the
  // operator has to approve. Click for the guidance; ✕ to stop showing it.
  let ghostsOpen = store.get("qhud.ghostsOpen") === "1";

  function renderPlaceholders(list) {
    for (const row of [...quotasEl.querySelectorAll(".q-row[data-pkey]")]) {
      row.remove();
    }
    quotasEl.querySelector(".q-more")?.remove();
    if (list.length === 0) {
      quotasEl.hidden = quotasEl.children.length === 0;
      return;
    }
    // One summary line instead of N rows: these accounts have no live
    // credential, so they carry no numbers and should not cost N lines of a
    // desktop widget. Click to expand.
    const more = el("div", "q-more");
    more.textContent = `${ghostsOpen ? "⌃" : "⌵"} ${list.length} account${
      list.length === 1 ? "" : "s"
    } need auth`;
    more.addEventListener("pointerdown", (e) => {
      e.stopPropagation();
      ghostsOpen = !ghostsOpen;
      store.set("qhud.ghostsOpen", ghostsOpen ? "1" : null);
      renderPlaceholders(list);
    });
    quotasEl.append(more);
    if (!ghostsOpen) {
      quotasEl.hidden = quotasEl.children.length === 0;
      return;
    }
    for (const p of list) {
      const row = el("div", "q-row q-ghost");
      row.dataset.pkey = `${p.provider}:${p.key}`;
      row.append(
        el("span", "q-prov", p.provider),
        el("span", "q-acct", p.label || p.key),
        el("span", "q-plan", p.plan || ""),
        el("span", "q-age", "needs re-auth"),
      );
      if (!p.plan) row.querySelector(".q-plan").hidden = true;
      const dismiss = el("button", "q-forget", "✕");
      dismiss.title = "stop showing this account";
      row.append(dismiss);
      row.title = `${p.label || p.key} — no stored credential.\n\n${
        p.hint || "sign in again to make it live"
      }`;
      const detail = el("div", "q-detail");
      detail.textContent = `no stored credential.\n${
        p.hint || "sign in again to make it live"
      }`;
      row.append(detail);
      quotasEl.append(row);
    }
    quotasEl.hidden = quotasEl.children.length === 0;
  }

  // Provider is the OUTER axis: it is what the operator picks when deciding
  // where to run the next task, it matches the pane vocabulary
  // (claude:1:main), and there are exactly three of them — so the widget's
  // scan path and height stay stable as accounts multiply.
  function providerHeader(provider) {
    let h = quotasEl.querySelector(`.q-sect[data-sect="${provider}"]`);
    if (!h) {
      h = el("div", "q-sect", provider);
      h.dataset.sect = provider;
    }
    return h;
  }

  // One vocabulary for one fact: every window is named by its duration —
  // 5H, 1D, 7D, 30D — everywhere. The wire says "weekly"/"30d" on codex
  // windows and the operator rightly asked whether "weekly" and "7D"
  // were different things; they are the same 7-day rolling window, so
  // they must carry the same name.
  const WIN_DISPLAY = {
    "5h": "5H",
    daily: "1D",
    weekly: "7D",
    "30d": "30D",
    yearly: "1Y",
  };
  const winLabel = (label) => WIN_DISPLAY[label] || label;

  // A scoped window's chip label: Claude's per-model weekly caps
  // (weekly_scoped) and any provider's non-primary pools (pool_<label>:
  // agy's 3p pools, codex's per-model limits). Unknown kinds return null
  // and stay tooltip-only.
  function scopedChipLabel(x) {
    if (!x.scope) return null;
    if (x.kind === "weekly_scoped") return `${x.scope} 7D`;
    if (x.kind?.startsWith("pool_"))
      return `${x.scope} ${winLabel(x.kind.slice(5))}`;
    return null;
  }

  // Scoped windows in the tooltip: on a PANE row this is the only place
  // they appear — a dated snapshot value rendered as a gauge next to live
  // ones reads as live (the v0.4.0 Fable bug: cache said 5%, truth 22%).
  function scopedTooltipLines(scoped, age) {
    return (scoped || [])
      .filter((x) => scopedChipLabel(x))
      .map(
        (x) =>
          `${scopedChipLabel(x)} ${x.pct}%${age ? ` (snapshot ~${age} old)` : ""}`,
      );
  }

  // Scoped gauges. Only for data whose provenance the row already wears:
  // an in-session refresh (live) or a whole-row snapshot with its age
  // caption. `stale` dims the chips (CLI-cache rows).
  function patchLiveScoped(row, scoped, stale) {
    const want = scoped.filter((x) => scopedChipLabel(x));
    const seen = new Set();
    for (const x of want) {
      const key = `m:${x.kind}:${x.scope}`;
      seen.add(key);
      let chip = row.querySelector(`[data-win="${CSS.escape(key)}"]`);
      if (!chip) {
        chip = buildChip(key, scopedChipLabel(x));
        row.append(chip);
      }
      chip.classList.toggle("stale", !!stale);
      patchQuotaChip(chip, { pct: x.pct, reset_unix: x.reset_unix });
    }
    for (const chip of [...row.querySelectorAll('[data-win^="m:"]')]) {
      if (!seen.has(chip.dataset.win)) chip.remove();
    }
  }

  function renderQuotas(quotas) {
    quotasEl.hidden = quotas.length === 0;
    const aliveProviders = new Set();
    const aliveRows = new Set();
    for (const q of quotas) {
      // One row per (provider, account) — a provider may carry several
      // accounts now (D-015), so the provider alone no longer keys a row.
      // (account, org) keys the row: one claude.ai account can hold a
      // team seat and a personal org — separate pools, separate rows.
      const acctKey =
        (q.account?.account_id || q.account?.email || "default") +
        (q.account?.org_id ? ` ${q.account.org_id}` : "");
      aliveProviders.add(q.provider);
      aliveRows.add(`${q.provider} ${acctKey}`);
      let row = quotasEl.querySelector(
        `.q-row[data-provider="${q.provider}"][data-acct="${CSS.escape(acctKey)}"]`,
      );
      if (!row) {
        row = buildQuotaRow(q.provider);
        row.dataset.acct = acctKey;
        // Insert after the provider's last existing row so a second
        // account never drags the section header out of place.
        const section = providerHeader(q.provider);
        if (!section.isConnected) quotasEl.append(section);
        let anchor = section;
        for (const r of quotasEl.querySelectorAll(
          `.q-row[data-provider="${q.provider}"]`,
        )) {
          anchor = r;
        }
        anchor.after(row);
      }
      // This account's slice of an in-session ⟳ (fetch_all returns one
      // entry per account); everything below prefers it over the payload.
      const liveFetch =
        q.provider === "claude" &&
        claudeFetch.state === "done" &&
        Array.isArray(claudeFetch.data)
          ? claudeFetch.data.find(
              // The fetch key IS the config dir (or "default"): with one
              // account in two orgs, account_id alone would feed one
              // org's numbers to both rows.
              (r) => r.key === (q.account?.config_dir || "default"),
            )
          : null;
      // The active codex workspace is merged into this row (its ↳ row
      // would be the same pool twice) — so the row must SAY which
      // workspace it is showing.
      const activeWs =
        q.provider === "codex"
          ? codexSource().rows.find(
              (w) =>
                w.active &&
                (!q.account?.account_id ||
                  w.account_id === q.account.account_id),
            )
          : null;
      // The agy RPC read and the codex active-workspace fetch have the
      // same snapshot shape as Claude's ⟳, so one override path serves
      // all three. Codex only counts when fetched THIS session — the
      // stored copy already reached this row through the backend merge,
      // dated.
      const liveUsage =
        liveFetch?.usage ||
        (q.provider === "agy" && agyFetch.state === "done"
          ? agyFetch.data
          : null) ||
        (activeWs && codexFetch.state === "done" ? wsToUsage(activeWs) : null);
      // The provider name lives in the section header now, so the row's own
      // slot carries the ACCOUNT — the thing that actually differs per row.
      row.querySelector(".q-prov").textContent = "";
      // Whose quota this is. Without it two logins on one provider are
      // indistinguishable; the chip stays empty when no account is known.
      const acctEl = row.querySelector(".q-acct");
      acctEl.textContent = q.account?.display || "";
      // "whose quota" now includes WHICH workspace of that login. Name
      // precedence matches the ↳ rows: the operator's registry mapping
      // first, then whatever the server supplied.
      const activeWsName = activeWs
        ? (state.payload?.workspace_names || {})[activeWs.account_id] ||
          activeWs.name
        : null;
      if (activeWsName && acctEl.textContent) {
        acctEl.textContent += ` · ${activeWsName}`;
      }
      acctEl.hidden = !acctEl.textContent;

      // Plan / seat, inline rather than tooltip-only: "whose quota and on
      // what plan" is the question the strip exists to answer, and hover
      // does not work on a keep-below widget you are not pointing at.
      const planEl = row.querySelector(".q-plan");
      const bits = [];
      if (q.account?.plan) {
        // An explicit plan string is already complete. Appending the derived
        // tier here is what produced "team (max_5x) (max_5x)".
        bits.push(q.account.plan);
      } else {
        if (q.account?.org_type) bits.push(shortPlan(q.account.org_type));
        for (const t of q.account?.tiers || []) {
          if (t.kind === "user") bits.push(`(${shortTier(t.tier)})`);
        }
      }

      planEl.textContent = bits.join(" ");
      planEl.hidden = bits.length === 0;

      // Usage credits beyond the plan ("extra usage") — the last thing the
      // provider's own usage page shows that the windows do not. A live ⟳
      // result outranks the payload's snapshot copy. Hidden entirely for
      // plans that never enabled it, so it does not tax every account line.
      let extra = q.extra;
      if (liveUsage?.extra) {
        extra = liveUsage.extra;
      }
      const extraEl = row.querySelector(".q-extra");
      const showExtra = !!extra && (extra.enabled || extra.used_minor > 0);
      extraEl.hidden = !showExtra;
      if (showExtra) {
        let txt = `extra ${fmtMinor(extra.used_minor, extra.exponent, extra.currency)}`;
        if (extra.limit_minor != null)
          txt += `/${fmtMinor(extra.limit_minor, extra.exponent, extra.currency)}`;
        extraEl.textContent = txt;
        extraEl.dataset.sev = extra.limit_reached
          ? "crit"
          : extra.percent != null
            ? sev(extra.percent)
            : "good";
      }

      // A cache-sourced row is NOT a live reading. Mark it on the row and
      // say how old it is, so a stale number can never pass for current.
      row.dataset.origin = q.origin || "pane";
      if (q.cache_fetched_at_ms) {
        const ageMin = Math.round((Date.now() - q.cache_fetched_at_ms) / 60000);
        row.dataset.cacheAge =
          ageMin < 90 ? `${ageMin}m` : `${Math.round(ageMin / 60)}h`;
      } else {
        delete row.dataset.cacheAge;
      }
      // Only a snapshot-ORIGIN row wears the age badge. On a live row the
      // snapshot merely contributes per-model windows, so badging it
      // would wrongly imply the visible gauges are stale. "cache" is the
      // CLI's own on-disk copy; "fetched" is qhud's last explicit ⟳.
      const ageEl = row.querySelector(".q-age");
      const snapshotRow = q.origin === "cache" || q.origin === "fetched";
      const showAge = snapshotRow && row.dataset.cacheAge;
      ageEl.textContent = !showAge
        ? ""
        : q.origin === "fetched"
          ? `⟳ ${row.dataset.cacheAge} ago`
          : `~${row.dataset.cacheAge} old`;
      ageEl.hidden = !showAge;

      const lines = [];
      if (q.origin === "cache") {
        lines.push(
          `no CLI running — from Claude's on-disk snapshot, ${row.dataset.cacheAge} old`,
        );
      } else if (q.origin === "fetched") {
        lines.push(
          `no CLI running — from qhud's last ⟳, ${row.dataset.cacheAge} ago`,
        );
      } else if (q.cache_fetched_at_ms) {
        lines.push(
          `per-model windows from snapshot, ${row.dataset.cacheAge} old`,
        );
      }
      lines.push(...scopedTooltipLines(q.scoped, row.dataset.cacheAge));
      if (showExtra) {
        lines.push(
          `extra usage: ${fmtMinor(extra.used_minor, extra.exponent, extra.currency)} used` +
            (extra.limit_minor != null
              ? ` of ${fmtMinor(extra.limit_minor, extra.exponent, extra.currency)}`
              : "") +
            (extra.percent != null ? ` (${extra.percent}%)` : "") +
            (extra.limit_reached ? " — spend limit reached" : ""),
        );
      }
      if (q.account) {
        const a = q.account;
        lines.push(
          `account: ${a.display || "unknown"}` +
            (a.email && a.email !== a.display ? ` <${a.email}>` : ""),
        );
        if (a.org)
          lines.push(`org: ${a.org}${a.org_type ? ` (${a.org_type})` : ""}`);
        // A team seat carries an org pool AND the member's own seat, each
        // with its own tier — collapsing them would hide a pool.
        for (const t of a.tiers || []) lines.push(`${t.kind} tier: ${t.tier}`);
      }
      if (activeWs) {
        lines.push(
          `workspace: ${activeWsName || activeWs.account_id.slice(0, 8)} (active login)`,
        );
        if (activeWs.plan_type)
          lines.push(`plan_type (wire): ${activeWs.plan_type}`);
        if (activeWs.credits_balance)
          lines.push(`credits ${activeWs.credits_balance}`);
      }
      if (q.from_label) {
        lines.push(
          `freshest reading: ${q.from_label}` +
            (q.session ? ` @${q.session}` : ""),
        );
      }
      row.title = lines.join("\n");
      // The same facts, inline: rows expand on selection because hover
      // tooltips barely exist for a keep-below widget.
      const detailEl = row.querySelector(".q-detail");
      if (detailEl) detailEl.textContent = lines.join("\n");
      // A live refresh outranks both the statusline and the cache: it is the
      // newest reading and the only one carrying per-model windows.
      let h5 = q.h5;
      let d7 = q.d7;
      let scoped = [];
      if (liveUsage) {
        if (liveUsage.five_hour)
          h5 = { ...liveUsage.five_hour, source: "live" };
        if (liveUsage.seven_day)
          d7 = { ...liveUsage.seven_day, source: "live" };
        scoped = liveUsage.scoped || [];
      } else if (q.origin === "fetched" || q.origin === "cache") {
        // The whole row is snapshot data wearing its age caption, so its
        // scoped windows may render as gauges too. On a PANE row they
        // stay tooltip-only: a day-old per-model value next to live
        // gauges reads as live (the v0.4.0 Fable bug).
        scoped = q.scoped || [];
      }
      patchQuotaChip(row.querySelector('[data-win="5h"]'), h5);
      patchQuotaChip(row.querySelector('[data-win="7d"]'), d7);
      patchLiveScoped(row, scoped, !liveUsage && q.origin === "cache");
      // A signed-in account with no numbers at all (an extra account never
      // ⟳'d, no cache): say what to do instead of rendering a bare name.
      if (q.provider === "claude" && !h5 && !d7 && !showAge) {
        ageEl.hidden = false;
        ageEl.textContent = "no data — ⟳";
      }
      const rowFetch =
        q.provider === "claude"
          ? claudeFetch
          : q.provider === "agy"
            ? agyFetch
            : null;
      if (rowFetch) {
        const b = row.querySelector(".q-refresh");
        if (b) {
          b.textContent = rowFetch.state === "loading" ? "…" : "⟳";
          b.classList.toggle("err", rowFetch.state === "error");
          if (rowFetch.error) b.title = rowFetch.error;
        }
      }
    }
    for (const row of [...quotasEl.children]) {
      if (row.dataset.pkey || row.dataset.wsid) continue; // owned by other renderers
      if (row.dataset.sect) {
        if (!aliveProviders.has(row.dataset.sect)) row.remove();
        continue;
      }
      if (
        row.dataset.provider &&
        !aliveRows.has(
          `${row.dataset.provider} ${row.dataset.acct || "default"}`,
        )
      )
        row.remove();
    }
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

    srcBadge.hidden = p.source !== "demo";

    patchRefreshAll();
    try {
      renderQuotas(p.quotas || []);
      // After the provider rows: workspace rows anchor to the codex row,
      // and before any in-session fetch they come from the payload's
      // stored copy.
      renderCodexExtra();
      renderPlaceholders(p.account_placeholders || []);
      // Positive confirmation that the strip painted. Absence of a JS error
      // is NOT proof the strip rendered — the pixels are unverifiable from
      // outside the webview (scrot cannot capture XWayland-composited
      // windows, D-010), so emit what was actually built.
      if (!window.__qhudStripLogged) {
        window.__qhudStripLogged = true;
        window.__TAURI__?.core?.invoke("ui_event", {
          event: `strip: ${quotasEl.querySelectorAll(".q-sect").length} sections, ${
            quotasEl.querySelectorAll(".q-row").length
          } rows, ${quotasEl.querySelectorAll(".q-chip").length} gauges (${
            quotasEl.querySelectorAll('[data-win^="m:"]').length
          } per-model), more=${quotasEl.querySelector(".q-more") ? 1 : 0}`,
        });
        logLabels("initial");
      }
    } catch (err) {
      reportJsError("renderQuotas", err);
    }

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
    for (const reset of quotasEl.querySelectorAll(
      ".q-reset[data-reset-unix]",
    )) {
      reset.textContent = fmtReset(Number(reset.dataset.resetUnix));
    }
    for (const sp of tilesEl.querySelectorAll(".elapsed[data-base-secs]")) {
      sp.textContent = fmtElapsed(Number(sp.dataset.baseSecs));
    }
  }, 1000);

  // ---- wiring ------------------------------------------------------

  // Frontend exceptions used to be invisible from outside the webview, so a
  // render-time throw looked identical to "the payload is fine" — that is how
  // a broken quota strip shipped once. Route them to stderr via ui_event.
  // Function declaration, not a const: it is referenced from render(),
  // which is defined earlier in the file.
  // The actual rendered text of every row. Counts cannot catch a duplicated
  // tier or a wire enum leaking into a label; this can. Fired again after the
  // Codex fetch, because workspace rows are appended later than initial render.
  function logLabels(tag) {
    try {
      const lines = [...quotasEl.querySelectorAll(".q-row")]
        .map((r) => {
          const ident = [".q-prov", ".q-acct", ".q-plan"]
            .map((sel) => r.querySelector(sel)?.textContent || "")
            .filter(Boolean)
            .join("|");
          // Gauge labels too: a wrong window NAME (weekly vs 7D) is
          // exactly the kind of bug counts cannot catch.
          const chips = [...r.querySelectorAll(".q-label")]
            .map((l) => l.textContent)
            .join(",");
          return ident + (chips ? ` [${chips}]` : "");
        })
        .filter(Boolean)
        .join("  /  ");
      window.__TAURI__?.core?.invoke("ui_event", {
        event: `labels(${tag}): ${lines}`,
      });
    } catch {}
  }

  function reportJsError(what, err) {
    const msg = `js-error ${what}: ${err && (err.stack || err.message || err)}`;
    try {
      window.__TAURI__?.core?.invoke("ui_event", { event: msg });
    } catch {}
    try {
      console.error(msg);
    } catch {}
  }
  window.addEventListener("error", (e) =>
    reportJsError("onerror", e.error || e.message),
  );
  window.addEventListener("unhandledrejection", (e) =>
    reportJsError("rejection", e.reason),
  );

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

    // Every real pointerdown, logged at the CAPTURE phase before any
    // handler can swallow it. This exists because "clicks do nothing"
    // has now been reported against a widget that passed every
    // compositor-path input test by the time it was investigated
    // (2026-08-12): the one measurement missing was where the operator's
    // actual click landed while it was broken. With this line, the next
    // report is diagnosable from stderr alone — a `ptr:` with a target
    // proves delivery and names what was hit; its absence proves the
    // event never reached the webview at all.
    document.addEventListener(
      "pointerdown",
      (e) => {
        const t = e.target;
        const tile = t.closest?.(".tile");
        const row = t.closest?.(".q-row");
        const btn = t.closest?.("button");
        const what = btn
          ? `${btn.className.split(" ")[0] || "button"}${
              row ? `@${row.dataset.provider || "?"}` : ""
            }`
          : tile
            ? `tile:${tile.dataset.paneId || "?"}`
            : row
              ? `row:${row.dataset.provider || row.dataset.pkey || row.dataset.wsid || "?"}`
              : String(t.id || t.className || t.tagName || "?").slice(0, 40);
        crumb(
          `ptr:${Math.round(e.clientX)},${Math.round(e.clientY)}:b${e.button}:${what}`,
        );
      },
      true,
    );

    // NOTE on freeze detection: a JS rAF-based watchdog lived here for
    // half a day and was removed — during a real presentation freeze
    // rAF kept firing (software rendering decouples it from the
    // screen), so it never saw what the operator saw. The watchdog
    // that actually measures the symptom (pixels) lives in the Rust
    // poll loop: frame_guard.rs.

    // Font size: Ctrl+wheel over the widget zooms the whole page
    // (pointer-only — the widget never takes keyboard focus), persisted
    // across restarts. Range 70–160%.
    const webview =
      tauri.webview && tauri.webview.getCurrentWebview
        ? tauri.webview.getCurrentWebview()
        : null;
    let zoom = parseFloat(store.get("qhud.zoom")) || 1;
    const applyZoom = (z) => {
      zoom = Math.min(1.6, Math.max(0.7, Math.round(z * 10) / 10));
      if (webview) webview.setZoom(zoom).catch(() => {});
      store.set("qhud.zoom", String(zoom));
      crumb("zoom:" + zoom.toFixed(1));
    };
    if (webview && zoom !== 1) applyZoom(zoom);
    window.addEventListener(
      "wheel",
      (e) => {
        if (!e.ctrlKey) return;
        e.preventDefault();
        applyZoom(zoom + (e.deltaY < 0 ? 0.1 : -0.1));
      },
      { passive: false },
    );

    // Layer peek indicator (tray "Pin above windows" / SIGUSR1).
    tauri.event.listen("qhud://layer", ({ payload }) => {
      metaEl.classList.toggle("pinned", !!payload);
    });

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
        if (e.target.closest(".grip, .refresh-all")) return;
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
      quotas: [
        {
          provider: "agy",
          from_label: "agy:1:research",
          session: "demo",
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
        {
          provider: "claude",
          from_label: "claude:1:main",
          session: "demo",
          h5: {
            pct: 88,
            source: "providerofficial",
            reset_unix: now + 2820,
            of_tokens: null,
          },
          d7: {
            pct: 31,
            source: "providerofficial",
            reset_unix: now + 363600,
            of_tokens: null,
          },
        },
        {
          provider: "codex",
          from_label: "codex:1:review",
          session: "demo",
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
      ],
      summary: { panes: 3, conflicts: 1, max_5h_pct: 88 },
      panes: [
        {
          pane_id: "%25",
          session: "demo",
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
          session: "demo",
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
          session: "demo",
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
    // Keep the preview "fresh" so the stale watchdog doesn't flag a
    // static page (screenshots and browser dev both use this path).
    setInterval(() => {
      state.payload.generated_at_ms = Date.now();
      state.receivedAt = Date.now();
      render();
    }, 2000);
  }
})();
