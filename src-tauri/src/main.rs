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
mod setup;
mod state;
mod updater;
mod window_watcher;

#[cfg(target_os = "linux")]
use std::env;

use state::AppState;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};

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
        .setup(move |app| setup::init(app, is_wayland))
        .invoke_handler(tauri::generate_handler![
            settings::validate_warframe_log_path,
            hotkeys::get_hotkey,
            hotkeys::set_hotkey,
            ocr::get_ocr_themes,
            updater::check_for_update,
            updater::download_and_relaunch_update,
            market::get_market_item,
            market::get_market_orders,
            market::get_market_statistics
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
