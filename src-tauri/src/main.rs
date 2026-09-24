#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod error;
mod hotkeys;
mod layer_shell;
mod market;
mod market_account;
mod market_notifications;
mod ocr;
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod relic_reward_capture;
#[cfg(target_os = "windows")]
mod relic_rewards;
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod relic_visual_detection;
mod setup;
mod state;
mod store_ext;
mod tray;
mod updater;
mod window_size;
mod window_watcher;
mod worldstate;

#[cfg(target_os = "linux")]
use std::env;

use state::AppState;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};

fn main() {
    // Initialize logging before anything else
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let is_wayland = apply_wayland_workarounds();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
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
        .on_window_event(|window, event| {
            window_size::handle_window_event(window, event);
            tray::handle_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            hotkeys::get_hotkey,
            hotkeys::set_hotkey,
            ocr::get_ocr_themes,
            updater::check_for_update,
            updater::download_and_relaunch_update,
            market::get_market_item,
            market::get_market_dictionary,
            market::get_most_traded_items,
            market::get_mastery_tradeable_prices,
            market::get_cached_market_items,
            market::get_market_orders,
            market::get_market_statistics,
            market_account::market_login,
            market_account::market_logout,
            market_account::market_session,
            market_account::market_set_status,
            market_account::market_top_orders,
            market_account::market_my_orders,
            market_account::market_item_details,
            market_account::market_create_listing,
            market_account::market_update_listing,
            market_account::market_delete_listing,
            market_notifications::start_market_notification_socket,
            market_notifications::stop_market_notification_socket,
            worldstate::get_world_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "linux")]
fn apply_wayland_workarounds() -> bool {
    let is_wayland = layer_shell::is_wayland_session();

    if is_wayland && env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // Safety: runs before Tauri starts and before any worker threads are spawned.
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
