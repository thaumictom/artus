mod dictionary;
mod hotkeys;
mod market_prices;
mod ocr;
mod state;

#[cfg(target_os = "linux")]
use std::env;

use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};

fn main() {
    let is_wayland = apply_wayland_workarounds();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            GlobalShortcutBuilder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        hotkeys::on_pressed(app, shortcut);
                    }
                })
                .build(),
        )
        .manage(AppState::default())
        .setup(move |app| {
            let overlay = app
                .get_webview_window("overlay")
                .ok_or("overlay window not found")?;

            overlay.hide()?;

            // Wayland compositors can reject these early input/focus mutations.
            if !is_wayland {
                let _ = overlay.set_ignore_cursor_events(true);
                let _ = overlay.set_focusable(false);
            }

            hotkeys::register_initial(app.handle())?;
            dictionary::refresh_dictionary_on_start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hotkeys::get_hotkey,
            hotkeys::set_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "linux")]
fn apply_wayland_workarounds() -> bool {
    let is_wayland = env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var("XDG_SESSION_TYPE")
            .map(|value| value.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false);

    if is_wayland && env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // Safety: this runs before the Tauri runtime starts and before any worker threads are spawned.
        unsafe {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    is_wayland
}

#[cfg(not(target_os = "linux"))]
fn apply_wayland_workarounds() -> bool {
    false
}
