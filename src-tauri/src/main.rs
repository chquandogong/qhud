#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod demo;
mod poll;
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

    tauri::Builder::default()
        // Single instance doubles as the peek IPC: `qhud --peek` from a
        // GNOME custom shortcut relays argv to the running widget and
        // exits; a plain re-launch is absorbed instead of stacking a
        // second widget.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|a| a == "--peek") {
                toggle_layer(app);
            } else {
                eprintln!("qhud: already running (second launch absorbed)");
            }
        }))
        .invoke_handler(tauri::generate_handler![ui_event])
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
