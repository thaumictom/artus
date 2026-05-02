use std::time::Duration;
use tauri::{AppHandle, Manager};
use active_win_pos_rs::get_active_window;
use sysinfo::System;
use std::sync::atomic::Ordering;

use crate::state::AppState;
use crate::hotkeys;

pub fn spawn_window_watcher(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        let mut ticks = 0;
        let mut was_focused = false;

        loop {
            std::thread::sleep(Duration::from_millis(200));
            ticks += 1;

            if ticks >= 25 {
                ticks = 0;
                sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                let is_running = sys.processes_by_exact_name("Warframe.x64.exe".as_ref()).count() > 0 ||
                                 sys.processes_by_exact_name("Warframe.exe".as_ref()).count() > 0;
                
                app_handle.state::<AppState>().warframe_running.store(is_running, Ordering::Release);
            }

            let mut is_focused = false;
            if let Ok(active_window) = get_active_window() {
                let app_name = active_window.app_name.to_lowercase();
                let title = active_window.title.to_lowercase();
                if app_name.contains("warframe") || title.contains("warframe") {
                    is_focused = true;
                }
            }

            let state = app_handle.state::<AppState>();
            if is_focused != was_focused {
                was_focused = is_focused;
                state.warframe_focused.store(is_focused, Ordering::Release);

                if is_focused {
                    hotkeys::register_all(&app_handle);
                } else {
                    hotkeys::unregister_all(&app_handle);
                    if let Some(overlay) = app_handle.get_webview_window("overlay") {
                        let _ = overlay.hide();
                    }
                }
            }
        }
    });
}
