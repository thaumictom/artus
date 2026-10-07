#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod data;
mod desktop;
mod error;
mod inventory;
mod market;
mod ocr;
mod relics;
mod state;
mod store_ext;

#[cfg(target_os = "linux")]
use std::env;

use state::AppState;
use tauri_plugin_global_shortcut::Builder as GlobalShortcutBuilder;

fn main() {
    // Initialize logging before anything else
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let is_wayland = apply_wayland_workarounds();

    tauri::Builder::default()
        // A second launch forwards to this instance before running any other setup.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            desktop::tray::show_artus(app);
        }))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_updater::Builder::new()
                .header("User-Agent", data::api::USER_AGENT)
                .expect("updater user agent")
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            GlobalShortcutBuilder::new()
                .with_handler(|app, shortcut, event| {
                    desktop::hotkeys::on_shortcut(app, shortcut, event.state);
                })
                .build(),
        )
        .manage(AppState::default())
        .manage(inventory::InventoryService::default())
        .setup(move |app| app::setup::init(app, is_wayland))
        .on_window_event(|window, event| {
            desktop::window_size::handle_window_event(window, event);
            desktop::tray::handle_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            inventory::inventory_snapshot,
            inventory::inventory_change_ocr_quantities,
            inventory::inventory_add_item,
            inventory::inventory_change_row_quantity,
            inventory::inventory_change_market_quantity,
            inventory::inventory_dismiss_new_items,
            inventory::inventory_reset,
            app::developer_console::set_developer_console_enabled,
            desktop::hotkeys::get_hotkey,
            desktop::hotkeys::set_hotkey,
            desktop::hotkeys::set_overlay_listing_dialog_open,
            ocr::get_ocr_themes,
            ocr::capture::show_relic_add_toast,
            #[cfg(target_os = "windows")]
            relics::auto_add::get_relic_selection_developer_image,
            app::updater::check_for_update,
            app::updater::download_and_relaunch_update,
            market::get_cached_market_item,
            market::get_cached_wfm_item,
            market::get_market_dictionary,
            market::get_most_traded_items,
            market::get_mastery_tradeable_prices,
            market::get_tradeable_today_statistics,
            market::get_cached_market_items,
            market::get_ocr_market_items,
            data::api::refresh_api_catalogs,
            data::api::get_api_catalogs_last_fetched,
            data::wiki_offerings::get_wiki_offerings,
            data::oracle::get_oracle_bounties,
            market::get_market_orders,
            market::get_market_statistics,
            market::account::market_login,
            market::account::market_logout,
            market::account::market_session,
            market::account::market_authenticated,
            market::account::market_set_status,
            market::account::market_schedule_invisible,
            market::account::market_top_orders,
            market::account::market_my_orders,
            market::account::market_item_details,
            market::account::market_create_listing,
            market::account::market_update_listing,
            market::account::market_set_listing_visibility,
            market::account::market_close_listing_one,
            market::account::market_delete_listing,
            market::notifications::start_market_notification_socket,
            market::notifications::stop_market_notification_socket,
            data::worldstate::get_world_state
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Err(error) = data::api::clear_session_caches(app) {
                    log::warn!("could not clear session API caches on exit: {error}");
                }
            }
        });
}

#[cfg(target_os = "linux")]
fn apply_wayland_workarounds() -> bool {
    let is_wayland = desktop::layer_shell::is_wayland_session();

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
