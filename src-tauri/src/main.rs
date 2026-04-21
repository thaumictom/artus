mod hotkeys;
mod layer_shell;
mod ocr;
mod state;

#[cfg(target_os = "linux")]
use std::env;

use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};
use tauri_plugin_updater::UpdaterExt;

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

            match ocr::load_ocr_dictionary(&app.handle()) {
                Ok(count) => println!("[ocr] loaded dictionary entries: {count}"),
                Err(err) => eprintln!("[ocr] failed to load OCR dictionary: {err}"),
            }

            match ocr::load_tradeable_item_prices(&app.handle()) {
                Ok(count) => println!("[ocr] loaded tradeable item prices: {count}"),
                Err(err) => eprintln!("[ocr] failed to load tradeable item prices: {err}"),
            }

            hotkeys::register_initial(app.handle())?;

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let Ok(updater) = app_handle.updater() else {
                    eprintln!("[updater] failed to create updater instance");
                    return;
                };

                match updater.check().await {
                    Ok(Some(update)) => {
                        println!("[updater] found update {}", update.version);

                        if let Err(err) = update.download_and_install(|_, _| {}, || {}).await {
                            eprintln!("[updater] failed to download/install update: {err}");
                            return;
                        }

                        println!("[updater] update installed, restarting app");
                        app_handle.restart();
                    }
                    Ok(None) => println!("[updater] no update available"),
                    Err(err) => eprintln!("[updater] failed to check for updates: {err}"),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            ocr::set_ocr_dictionary_match_threshold
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
