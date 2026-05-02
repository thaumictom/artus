//! Application initialization — runs once during Tauri's `setup` hook.

use log::{error, info};
use tauri::{App, Manager};

use crate::error::AppResult;
use crate::{hotkeys, layer_shell, ocr, relic_rewards, window_watcher};
use crate::state::AppState;

/// Called by Tauri during startup to configure windows, load data, and spawn
/// background tasks.
pub fn init(app: &mut App, is_wayland: bool) -> Result<(), Box<dyn std::error::Error>> {
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
        let _ = overlay.set_focusable(false);
    }

    // Load OCR theme colors from embedded TOML
    match ocr::load_primary_theme_options(&app.handle()) {
        Ok(themes) => {
            let map = themes.into_iter().map(|t| (t.name, t.rgb)).collect();
            if let Ok(mut colors) = app.state::<AppState>().ocr_theme_colors.lock() {
                *colors = map;
            }
        }
        Err(err) => error!("failed to load primary themes: {err}"),
    }

    // Fetch remote dictionary and price data
    log_result("dictionary entries", ocr::load_ocr_dictionary(&app.handle()));
    log_result(
        "tradeable item prices",
        ocr::load_tradeable_item_prices(&app.handle()),
    );

    // Spawn background tasks
    relic_rewards::spawn_log_tailer(app.handle().clone());
    hotkeys::register_initial(app.handle())?;
    window_watcher::spawn_window_watcher(app.handle().clone());

    Ok(())
}

/// Logs the result of a data-loading operation.
fn log_result(label: &str, result: AppResult<usize>) {
    match result {
        Ok(count) => info!("loaded {label}: {count}"),
        Err(err) => error!("failed to load {label}: {err}"),
    }
}
