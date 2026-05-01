use tauri::{App, Manager};
use tauri_plugin_store::StoreExt;

use crate::{layer_shell, ocr, relic_rewards, hotkeys};
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
}
