#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod demo;
mod poll;
mod view;

use tauri::Manager;

/// Stderr breadcrumb for UI interactions. Invisible in the UI, but
/// greppable in logs/journal — real-input verification depends on it
/// (window-title beacons pollute WM_NAME; see D-010).
#[tauri::command]
fn ui_event(event: String) {
    eprintln!("qhud ui: {event}");
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

    tauri::Builder::default()
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
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let toggle = MenuItemBuilder::with_id("toggle", "Show / Hide").build(app)?;
    // Recovery affordance (cross-validation CV-1): a keep-below window
    // restored onto a disconnected monitor has no visible handle, so
    // the tray must be able to pull it back.
    let reset = MenuItemBuilder::with_id("reset", "Reset position").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit qhud").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&toggle, &reset, &quit])
        .build()?;

    let mut builder = TrayIconBuilder::with_id("qhud-tray")
        .menu(&menu)
        .tooltip("qhud — AI CLI HUD");
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => {
                if let Some(win) = app.get_webview_window("main") {
                    let visible = win.is_visible().unwrap_or(true);
                    let _ = if visible { win.hide() } else { win.show() };
                }
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
