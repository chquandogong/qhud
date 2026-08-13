#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod agy_usage;
mod claude_usage;
mod codex_usage;
mod demo;
mod fetched_store;
mod poll;
mod registry;
mod usage_cache;
mod view;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Emitter, Manager};

/// Stderr breadcrumb for UI interactions. Invisible in the UI, but
/// greppable in logs/journal — real-input verification depends on it
/// (window-title beacons pollute WM_NAME; see D-010).
#[tauri::command]
fn ui_event(event: String) {
    eprintln!("qhud ui: {event}");
}

/// Fetches every Codex workspace's quota, on explicit operator request.
///
/// The ONLY outbound network call in qhud, and it is never on a timer —
/// the poll loop stays passive. Reads the access token already on disk and
/// never runs the refresh grant (Codex refresh tokens are single-use and
/// rotated; a failed write-back would break `codex login`).
#[tauri::command]
async fn fetch_codex_workspaces() -> Result<Vec<codex_usage::WorkspaceUsage>, String> {
    eprintln!("qhud ui: codex-workspace-fetch requested");
    let out = codex_usage::fetch_all_workspaces().await;
    match &out {
        Ok(w) => {
            eprintln!("qhud: codex fetch ok ({} workspaces)", w.len());
            // Persisted so the rows survive a restart, dated (fetched_store).
            fetched_store::record_codex(w, view::now_ms());
        }
        Err(e) => eprintln!("qhud: codex fetch failed: {e}"),
    }
    out
}

/// Explicit refresh for Claude's per-model usage windows.
///
/// The one place qhud sends a credential off the machine, and only from a
/// click: the statusLine feed has no per-model windows, and nothing qhud can
/// run refreshes ~/.claude.json's cache (verified: --version, doctor, mcp list,
/// and a real headless --print all leave fetchedAtMs untouched). Never on a
/// timer, and never runs the OAuth refresh grant.
#[tauri::command]
async fn fetch_claude_usage() -> Result<Vec<claude_usage::AccountFetch>, String> {
    eprintln!("qhud ui: claude-usage-refresh requested");
    // fetch_all records each success to the fetched store itself.
    let out = claude_usage::fetch_all(view::now_ms()).await;
    match &out {
        Ok(accounts) => {
            for a in accounts {
                eprintln!(
                    "qhud: claude usage ok [{}] (5h {:?}, 7d {:?}, {} scoped)",
                    a.key,
                    a.usage.five_hour.as_ref().map(|w| w.pct),
                    a.usage.seven_day.as_ref().map(|w| w.pct),
                    a.usage.scoped.len()
                );
            }
        }
        Err(e) => eprintln!("qhud: claude usage failed: {e}"),
    }
    out
}

/// agy quota via the CLI's own loopback Connect RPC — no token, and
/// nothing leaves the machine (the agy process owns its auth). Only
/// answers while agy runs; the fetched store keeps the last read for
/// when it does not.
#[tauri::command]
async fn fetch_agy_usage() -> Result<crate::usage_cache::CachedUsage, String> {
    eprintln!("qhud ui: agy-usage-refresh requested");
    let out = agy_usage::fetch(view::now_ms()).await;
    match &out {
        Ok(u) => {
            eprintln!(
                "qhud: agy usage ok (5h {:?}, 7d {:?}, {} pools)",
                u.five_hour.as_ref().map(|w| w.pct),
                u.seven_day.as_ref().map(|w| w.pct),
                u.scoped.len()
            );
            fetched_store::record_agy(u);
        }
        Err(e) => eprintln!("qhud: agy usage failed: {e}"),
    }
    out
}

/// Frame-clock self-heal, last rung: re-exec the widget in place. The
/// window-state plugin restores geometry and the fetched store lives on
/// disk, so the only cost is one blink — better than a wallpaper widget
/// frozen on an hours-old frame until a human notices. The child gets
/// --respawned so it waits out this process before the single-instance
/// guard runs.
#[tauri::command]
fn restart_self(app: tauri::AppHandle) {
    eprintln!("qhud ui: framestall restart — re-exec");
    if let Ok(exe) = std::env::current_exe() {
        match std::process::Command::new(exe).arg("--respawned").spawn() {
            Ok(_) => app.exit(0),
            Err(e) => eprintln!("qhud: self-restart spawn failed: {e}"),
        }
    }
}

/// Records "I no longer use this account" so its placeholder stops
/// appearing. Only suppresses placeholders — a live credential always
/// shows, because silently hiding an account in active use is worse than
/// showing one the operator tried to dismiss (see registry rule 1).
#[tauri::command]
fn forget_account(provider: String, key: String) -> Result<(), String> {
    eprintln!("qhud ui: forget-account {provider}:{key}");
    registry::forget_and_save(&provider, &key)
}

/// Current layer: false = desktop layer (keep-below, the default),
/// true = pinned above windows for a peek (D-012).
static PINNED: AtomicBool = AtomicBool::new(false);

fn apply_layer(app: &tauri::AppHandle, pinned: bool) {
    if let Some(win) = app.get_webview_window("main") {
        if pinned {
            let _ = win.set_always_on_bottom(false);
            let _ = win.set_always_on_top(true);
        } else {
            let _ = win.set_always_on_top(false);
            let _ = win.set_always_on_bottom(true);
        }
        // Layer flips can shed the widget states on X11 — re-assert.
        let _ = win.set_visible_on_all_workspaces(true);
        let _ = win.set_skip_taskbar(true);
        let _ = win.show();
    }
    let _ = app.emit("qhud://layer", pinned);
    eprintln!("qhud ui: layer:{}", if pinned { "pinned" } else { "below" });
}

/// Toggles between the desktop layer and pinned-above. Reachable from
/// the tray and from `qhud --peek` (second-instance argv relay) so a
/// GNOME custom keyboard shortcut can peek the widget — app-global
/// hotkeys are not available to XWayland clients on Wayland, and Unix
/// signals are off-limits: WebKitGTK's JavaScriptCore reserves SIGUSR1
/// for thread suspension, so hooking it segfaults the webview (D-012).
fn toggle_layer(app: &tauri::AppHandle) -> bool {
    let pinned = !PINNED.load(Ordering::Relaxed);
    PINNED.store(pinned, Ordering::Relaxed);
    apply_layer(app, pinned);
    pinned
}

fn main() {
    // GNOME on Wayland exposes no layer-shell to third-party apps, so
    // the desktop-widget layer (keep-below + sticky) and global window
    // positioning only exist through XWayland. Force the X11 GDK
    // backend before any GTK code runs. X11 sessions are unaffected;
    // set QHUD_NO_X11_FORCE=1 to opt out (e.g. on wlroots compositors
    // where you prefer native Wayland and manage layering yourself).
    // SAFETY: runs before any other thread is spawned.
    unsafe {
        if std::env::var_os("GDK_BACKEND").is_none()
            && std::env::var_os("QHUD_NO_X11_FORCE").is_none()
        {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        // WebKitGTK's DMABUF renderer froze this widget's output after
        // display power cycles: proven 2026-08-13 with the window pixmap
        // byte-identical across seconds while JS, input and IPC all ran
        // (sel:/qsel:/fetch breadcrumbs firing) — the operator saw an
        // hours-old frame and read it as "selection doesn't work". The
        // GPU path was degraded from launch on this stack (libEGL "DRI3
        // device" errors), and a wallpaper widget must survive overnight
        // DPMS. The SHM path renders this small strip effortlessly.
        // Opt out with QHUD_KEEP_DMABUF=1.
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
            && std::env::var_os("QHUD_KEEP_DMABUF").is_none()
        {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        // …and DMABUF-off alone proved insufficient: the freeze recurred
        // the same day at an idle blank (2026-08-13 11:51) with the
        // variable confirmed propagated to the WebKit child. The frozen
        // instance showed WebKit's VBlankMonitor waiting on a DRM vblank
        // — the threaded-compositor frame clock is the fragile piece, so
        // take the whole accelerated-compositing path out. Software
        // rendering is effortless for a strip this size. Opt out with
        // QHUD_KEEP_COMPOSITING=1. (Belt: the frontend also carries a
        // frame-clock watchdog that jiggles, then re-execs — see app.js.)
        if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none()
            && std::env::var_os("QHUD_KEEP_COMPOSITING").is_none()
        {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    // Respawn handoff (frame-stall self-heal, last rung): the fresh
    // process waits out the dying one so the single-instance guard does
    // not absorb the replacement into the instance that is exiting.
    if std::env::args().any(|a| a == "--respawned") {
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    // Diagnostic mode (before single-instance, no GTK): print one
    // observe payload — exactly what the widget renders — and exit.
    if std::env::args().any(|a| a == "--dump") {
        match poll::dump_once() {
            Some(json) => println!("{json}"),
            None => {
                eprintln!("qhud: no live mux source — demo payload follows");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&demo::payload()).unwrap_or_default()
                );
            }
        }
        return;
    }

    // Diagnostic twin of the CODEX row click: runs the exact same
    // fetch_all_workspaces() the UI invokes, so the network path can be
    // verified without synthesizing pointer input into a keep-below widget
    // (D-010: real input must go through the compositor).
    if std::env::args().any(|a| a == "--claude-usage") {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("qhud: runtime: {e}");
                return;
            }
        };
        // The CLI twin runs exactly what the ⟳ click does — fetch_all
        // records each account to the store, so a GNOME-shortcut refresh
        // feeds the widget's next tick too.
        match rt.block_on(claude_usage::fetch_all(view::now_ms())) {
            Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default()),
            Err(e) => eprintln!("qhud: claude usage failed: {e}"),
        }
        return;
    }

    if std::env::args().any(|a| a == "--agy-usage") {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("qhud: runtime: {e}");
                return;
            }
        };
        match rt.block_on(agy_usage::fetch(view::now_ms())) {
            Ok(u) => {
                fetched_store::record_agy(&u);
                println!("{}", serde_json::to_string_pretty(&u).unwrap_or_default());
            }
            Err(e) => eprintln!("qhud: agy usage failed: {e}"),
        }
        return;
    }

    // Diagnostic for the app-server fallback alone (FR-18): the path only
    // fires inside --codex-usage when the raw HTTP path fails, which makes
    // it untestable on demand without this.
    if std::env::args().any(|a| a == "--codex-appserver") {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("qhud: runtime: {e}");
                return;
            }
        };
        match rt.block_on(codex_usage::fetch_via_app_server()) {
            Ok(w) => println!("{}", serde_json::to_string_pretty(&w).unwrap_or_default()),
            Err(e) => eprintln!("qhud: codex app-server failed: {e}"),
        }
        return;
    }

    if std::env::args().any(|a| a == "--codex-usage") {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("qhud: runtime: {e}");
                return;
            }
        };
        match rt.block_on(codex_usage::fetch_all_workspaces()) {
            Ok(w) => {
                fetched_store::record_codex(&w, view::now_ms());
                println!("{}", serde_json::to_string_pretty(&w).unwrap_or_default());
            }
            Err(e) => eprintln!("qhud: codex usage failed: {e}"),
        }
        return;
    }

    tauri::Builder::default()
        // Single instance doubles as the peek IPC: `qhud --peek` from a
        // GNOME custom shortcut relays argv to the running widget and
        // exits; a plain re-launch is absorbed instead of stacking a
        // second widget.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|a| a == "--peek") {
                toggle_layer(app);
            } else if argv.iter().any(|a| a == "--refresh-claude") {
                let _ = app.emit("qhud://refresh-claude", ());
                eprintln!("qhud ui: refresh-claude relayed");
            } else if argv.iter().any(|a| a == "--fetch-codex") {
                // Same argv-relay trick as --peek (D-012): pointer input
                // cannot be synthesized into a keep-below widget, so the
                // click-only Codex fetch needs a non-pointer trigger. Also
                // bindable to a GNOME shortcut.
                let _ = app.emit("qhud://fetch-codex", ());
                eprintln!("qhud ui: fetch-codex relayed");
            } else if argv.iter().any(|a| a == "--refresh-all") {
                // One gesture, every provider — the topbar ⟳'s twin.
                let _ = app.emit("qhud://refresh-all", ());
                eprintln!("qhud ui: refresh-all relayed");
            } else {
                eprintln!("qhud: already running (second launch absorbed)");
            }
        }))
        .invoke_handler(tauri::generate_handler![
            ui_event,
            fetch_codex_workspaces,
            fetch_claude_usage,
            fetch_agy_usage,
            forget_account,
            restart_self
        ])
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let win = app
                .get_webview_window("main")
                .expect("main window is defined in tauri.conf.json");
            // tauri.conf.json requests these too, but X11 window states
            // only stick once the window is realized — re-assert here.
            let _ = win.set_always_on_bottom(true);
            let _ = win.set_visible_on_all_workspaces(true);
            let _ = win.set_skip_taskbar(true);

            if let Err(e) = tray(app) {
                eprintln!("qhud: tray unavailable ({e}); continuing without it");
            }

            let handle = app.handle().clone();
            std::thread::spawn(move || poll::run(handle));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("qhud failed to start");
}

fn tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let toggle = MenuItemBuilder::with_id("toggle", "Show / Hide").build(app)?;
    // Peek affordance (D-012): a keep-below widget is often covered;
    // this flips it above every window and back.
    let pin = CheckMenuItemBuilder::with_id("pin", "Pin above windows")
        .checked(false)
        .build(app)?;
    // Recovery affordance (cross-validation CV-1): a keep-below window
    // restored onto a disconnected monitor has no visible handle, so
    // the tray must be able to pull it back.
    let reset = MenuItemBuilder::with_id("reset", "Reset position").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit qhud").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&toggle, &pin, &reset, &quit])
        .build()?;

    let mut builder = TrayIconBuilder::with_id("qhud-tray")
        .menu(&menu)
        .tooltip("qhud — AI CLI HUD");
    // Light symbolic-style glyph: the app tile is dark and disappears
    // against GNOME's dark top bar.
    match tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png")) {
        Ok(icon) => builder = builder.icon(icon),
        Err(_) => {
            if let Some(icon) = app.default_window_icon() {
                builder = builder.icon(icon.clone());
            }
        }
    }
    let pin_item = pin.clone();
    builder
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => {
                if let Some(win) = app.get_webview_window("main") {
                    let visible = win.is_visible().unwrap_or(true);
                    let _ = if visible { win.hide() } else { win.show() };
                }
            }
            "pin" => {
                let pinned = toggle_layer(app);
                let _ = pin_item.set_checked(pinned);
            }
            "reset" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_position(tauri::PhysicalPosition::new(50, 50));
                    let _ = win.show();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
