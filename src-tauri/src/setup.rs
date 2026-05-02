use tauri::{App, Manager};
use tauri_plugin_store::StoreExt;

use crate::{layer_shell, ocr, relic_rewards, hotkeys, window_watcher};
use crate::state::AppState;

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
        // Keep the overlay non-focusable on Wayland; click-through is re-applied when shown.
        let _ = overlay.set_focusable(false);
    }

    match ocr::load_primary_theme_options(&app.handle()) {
        Ok(themes) => {
            let mut map = std::collections::HashMap::new();
            for theme in themes {
                map.insert(theme.name, theme.rgb);
            }
            if let Ok(mut theme_colors) = app.state::<AppState>().ocr_theme_colors.lock() {
                *theme_colors = map;
            }
        }
        Err(err) => eprintln!("[ocr] failed to load primary themes: {err}"),
    }

    match ocr::load_ocr_dictionary(&app.handle()) {
        Ok(count) => println!("[ocr] loaded dictionary entries: {count}"),
        Err(err) => eprintln!("[ocr] failed to load OCR dictionary: {err}"),
    }

    match ocr::load_tradeable_item_prices(&app.handle()) {
        Ok(count) => println!("[ocr] loaded tradeable item prices: {count}"),
        Err(err) => eprintln!("[ocr] failed to load tradeable item prices: {err}"),
    }

    relic_rewards::spawn_log_tailer(app.handle().clone());

    hotkeys::register_initial(app.handle())?;

    window_watcher::spawn_window_watcher(app.handle().clone());

    Ok(())
}
