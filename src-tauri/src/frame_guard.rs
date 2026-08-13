//! Pixel-level frame watchdog.
//!
//! Three renderer-layer bets failed to stop the display-sleep freeze
//! (DMABUF off, compositing off, a JS rAF watchdog that stayed blind
//! because rAF keeps firing in software mode while nothing reaches the
//! screen). So this guard measures the SYMPTOM itself: it hashes a strip
//! of the widget's own window every ~28 s. The footer clock there
//! repaints every second, so two identical consecutive hashes mean the
//! client has not painted for half a minute — frozen, no matter which
//! layer wedged.
//!
//! Heal ladder, each rung proven or bounded:
//!  1. unmap/remap (hide+show) — verified live on a frozen instance
//!     (2026-08-13): remapping resets the compositor's frame tracking
//!     and painting resumes. Layer states are re-asserted after.
//!  2. re-exec (--respawned) if the next sample is still frozen.
//!
//! Silent while healthy; every detection and heal leaves a stderr line.

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

use tauri::Manager;

static LAST_HASH: AtomicU64 = AtomicU64::new(0);
static PHASE: AtomicU8 = AtomicU8::new(0);

/// What the state machine wants done after a sample.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Action {
    /// Painting normally (or first sample).
    None,
    /// One frozen interval: unmap/remap the window.
    Remap,
    /// Still frozen after the remap: re-exec the widget.
    Restart,
}

/// Pure decision: compare this sample against the previous one.
/// `prev == 0` means "no sample yet" (FNV never yields 0 on real input).
pub fn decide(prev: u64, cur: u64, phase: u8) -> (Action, u8) {
    if prev == 0 || cur != prev {
        return (Action::None, 0);
    }
    match phase {
        0 => (Action::Remap, 1),
        1 => (Action::Restart, 2),
        _ => (Action::None, phase),
    }
}

/// FNV-1a, 64 bit — stable, dependency-free, plenty for change detection.
pub fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // Reserve 0 as the "no sample" sentinel.
    if hash == 0 { 1 } else { hash }
}

/// Called from the poll loop every tick; samples every 14th (~28 s).
/// All GTK/GDK access happens on the main thread.
pub fn tick(app: &tauri::AppHandle, tick: u64) {
    if !tick.is_multiple_of(14) {
        return;
    }
    let app = app.clone();
    let _ = app.clone().run_on_main_thread(move || sample(&app));
}

fn sample(app: &tauri::AppHandle) {
    use gtk::gdk::prelude::*;
    use gtk::prelude::*;

    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    // Hidden via tray: static pixels are legitimate. Reset instead of
    // judging.
    if !win.is_visible().unwrap_or(false) {
        LAST_HASH.store(0, Ordering::Relaxed);
        PHASE.store(0, Ordering::Relaxed);
        return;
    }
    let Ok(gtk_win) = win.gtk_window() else {
        return;
    };
    let Some(gdk_win) = gtk_win.window() else {
        return;
    };
    let (w, h) = (gdk_win.width(), gdk_win.height());
    if w < 60 || h < 60 {
        return;
    }
    // The footer strip: `updated HH:MM:SS` repaints every second.
    // (gdk_pixbuf_get_from_window, bound as WindowExtManual::pixbuf.)
    let Some(pix) = gdk_win.pixbuf(0, h - 28, w.min(420), 24) else {
        return;
    };
    let cur = fnv64(pix.read_pixel_bytes().as_ref());

    let prev = LAST_HASH.swap(cur, Ordering::Relaxed);
    if prev == 0 {
        // Once per arm-up: proves the pixbuf sampling path works on this
        // system at all — a silently failing sampler would be a watchdog
        // that never barks.
        eprintln!("qhud: frame guard armed (first pixel sample ok)");
    }
    let phase = PHASE.load(Ordering::Relaxed);
    let (action, next_phase) = decide(prev, cur, phase);
    PHASE.store(next_phase, Ordering::Relaxed);

    match action {
        Action::None => {}
        Action::Remap => {
            eprintln!("qhud: frame freeze detected (footer static ~28s) — remap heal");
            let _ = win.hide();
            let _ = win.show();
            crate::reassert_layer(app);
        }
        Action::Restart => {
            eprintln!("qhud: frame freeze survived remap — re-exec");
            crate::respawn(app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv64_is_stable_and_input_sensitive() {
        assert_eq!(fnv64(b"footer"), fnv64(b"footer"));
        assert_ne!(fnv64(b"12:44:07"), fnv64(b"12:44:08"));
        assert_ne!(fnv64(b""), 0, "0 stays reserved as the sentinel");
    }

    #[test]
    fn changing_pixels_never_trigger_and_reset_the_ladder() {
        assert_eq!(decide(0, 42, 0), (Action::None, 0), "first sample");
        assert_eq!(decide(41, 42, 0), (Action::None, 0));
        // A heal ladder in progress is abandoned the moment paint resumes.
        assert_eq!(decide(41, 42, 1), (Action::None, 0));
    }

    #[test]
    fn frozen_pixels_climb_remap_then_restart() {
        let (a1, p1) = decide(42, 42, 0);
        assert_eq!(a1, Action::Remap);
        let (a2, p2) = decide(42, 42, p1);
        assert_eq!(a2, Action::Restart);
        // Restart is in flight; do not spam further actions.
        assert_eq!(decide(42, 42, p2).0, Action::None);
    }
}
