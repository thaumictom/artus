//! Warframe process and window-focus watcher.
//!
//! Polls the system every [`POLL_INTERVAL`] to detect whether Warframe is
//! the active foreground window. On focus transitions, hotkeys are
//! registered/unregistered and the overlay is hidden on blur.

use std::sync::atomic::Ordering;
use std::time::Duration;

use active_win_pos_rs::get_active_window;
use log::info;
use sysinfo::System;
use tauri::{AppHandle, Manager};

use crate::hotkeys;
use crate::state::AppState;

// ── Configuration ─────────────────────────────────────────────────────────────

/// How often to check window focus (200ms).
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Process-list refresh happens every N ticks (25 × 200ms = 5s).
const PROCESS_CHECK_TICKS: u32 = 25;

/// Executable names to look for when checking if Warframe is running.
const WARFRAME_PROCESS_NAMES: [&str; 2] = ["Warframe.x64.exe", "Warframe.exe"];

// ── Public API ────────────────────────────────────────────────────────────────

/// Spawns an async task that monitors Warframe focus and process state.
///
/// Note: `get_active_window()` and `System::refresh_processes()` are brief
/// blocking calls. They run on the async runtime directly because the poll
/// interval is 200ms and these calls typically complete in <20ms.
pub fn spawn_window_watcher(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new_all();
        let mut ticks: u32 = 0;
        let mut was_focused = false;

        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            ticks += 1;

            // Periodically refresh the process list to detect Warframe launch/exit
            if ticks >= PROCESS_CHECK_TICKS {
                ticks = 0;
                sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                let running = is_warframe_running(&sys);
                app_handle
                    .state::<AppState>()
                    .warframe_running
                    .store(running, Ordering::Release);
            }

            let focused = is_warframe_focused();
            if focused == was_focused {
                continue;
            }

            // Focus transition detected
            was_focused = focused;
            app_handle
                .state::<AppState>()
                .warframe_focused
                .store(focused, Ordering::Release);

            if focused {
                info!("Warframe gained focus — registering hotkeys");
                hotkeys::register_all(&app_handle);
            } else {
                info!("Warframe lost focus — unregistering hotkeys, hiding overlay");
                hotkeys::unregister_all(&app_handle);
                if let Some(overlay) = app_handle.get_webview_window("overlay") {
                    let _ = overlay.hide();
                }
            }
        }
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Checks if the currently focused window belongs to Warframe.
fn is_warframe_focused() -> bool {
    get_active_window()
        .map(|w| {
            let name = w.app_name.to_lowercase();
            let title = w.title.to_lowercase();
            name.contains("warframe") || title.contains("warframe")
        })
        .unwrap_or(false)
}

/// Checks if any Warframe process is present in the system process list.
/// Uses `.next().is_some()` to short-circuit instead of `.count() > 0`.
fn is_warframe_running(sys: &System) -> bool {
    WARFRAME_PROCESS_NAMES
        .iter()
        .any(|name| sys.processes_by_exact_name(name.as_ref()).next().is_some())
}
