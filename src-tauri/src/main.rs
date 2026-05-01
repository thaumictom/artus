#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod hotkeys;
mod layer_shell;
mod market;
mod ocr;
mod relic_rewards;
mod settings;
mod state;
mod updater;

#[cfg(target_os = "linux")]
use std::env;

use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};
use tauri_plugin_store::StoreExt;

fn main() {
    let is_wayland = apply_wayland_workarounds();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
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

            let layer_shell_enabled = layer_shell::configure_overlay_window(&overlay)?;

            // Apply input pass-through eagerly on non-Wayland.
            if !is_wayland {
                let _ = overlay.set_ignore_cursor_events(true);
                let _ = overlay.set_focusable(false);
            } else if layer_shell_enabled {
                // Keep the overlay non-focusable on Wayland; click-through is re-applied when shown.
                let _ = overlay.set_focusable(false);
            }

            if let Err(err) = ocr::load_persisted_overlay_duration(&app.handle()) {
                eprintln!("[ocr] failed to load persisted overlay duration: {err}");
            }

            if let Err(err) = ocr::load_persisted_overlay_mode(&app.handle()) {
                eprintln!("[ocr] failed to load persisted overlay mode: {err}");
            }

            if let Err(err) = ocr::load_persisted_ocr_dictionary_mapping_settings(&app.handle()) {
                eprintln!("[ocr] failed to load persisted OCR dictionary mapping settings: {err}");
            }

            if let Err(err) = ocr::load_persisted_ocr_theme(&app.handle()) {
                eprintln!("[ocr] failed to load persisted OCR theme: {err}");
            }

            match ocr::load_ocr_dictionary(&app.handle()) {
                Ok(count) => println!("[ocr] loaded dictionary entries: {count}"),
                Err(err) => eprintln!("[ocr] failed to load OCR dictionary: {err}"),
            }

            match ocr::load_tradeable_item_prices(&app.handle()) {
                Ok(count) => println!("[ocr] loaded tradeable item prices: {count}"),
                Err(err) => eprintln!("[ocr] failed to load tradeable item prices: {err}"),
            }

            // Load persisted settings
            if let Ok(store) = app.handle().store("settings.json") {
                let state = app.state::<AppState>();

                if let Some(enabled) = store
                    .get("relic_reward_detection")
                    .and_then(|v| v.as_bool())
                {
                    let mut relic_reward_detection = state.relic_reward_detection.lock().unwrap();
                    *relic_reward_detection = enabled;
                }

                if let Some(path) = store
                    .get("warframe_log_path")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                {
                    let mut warframe_log_path = state.warframe_log_path.lock().unwrap();
                    *warframe_log_path = path;
                }
            }

            relic_rewards::spawn_log_tailer(app.handle().clone());

            hotkeys::register_initial(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::patch_settings,
            hotkeys::get_hotkey,
            hotkeys::set_hotkey,
            ocr::get_ocr_theme_settings,
            ocr::set_ocr_theme,
            ocr::get_overlay_duration_secs,
            ocr::set_overlay_duration_secs,
            ocr::get_overlay_toggle_mode,
            ocr::set_overlay_toggle_mode,
            ocr::get_ocr_dictionary_mapping_settings,
            ocr::set_ocr_dictionary_mapping_enabled,
            ocr::set_ocr_dictionary_match_threshold,
            updater::check_for_update,
            updater::download_and_relaunch_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "linux")]
fn apply_wayland_workarounds() -> bool {
    let is_wayland = layer_shell::is_wayland_session();

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
