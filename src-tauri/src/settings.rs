use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::ocr;
use crate::state::AppState;

const WARFRAME_LOG_FILENAME: &str = "EE.log";
const SETTINGS_STORE_PATH: &str = "settings.json";

#[derive(Debug, Clone, Serialize)]
pub struct SettingsPayload {
    pub ocr_theme: ocr::OcrThemeSettingsPayload,
    pub overlay_duration_secs: u64,
    pub overlay_toggle_mode: bool,
    pub ocr_dictionary_mapping: ocr::OcrDictionaryMappingSettingsPayload,
    pub warframe_log_path: String,
    pub relic_reward_detection: bool,
    pub show_ocr_bounding_boxes: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SettingsPatchPayload {
    pub ocr_theme: Option<String>,
    pub overlay_duration_secs: Option<u64>,
    pub overlay_toggle_mode: Option<bool>,
    pub ocr_dictionary_mapping_enabled: Option<bool>,
    pub ocr_dictionary_match_threshold: Option<f64>,
    pub warframe_log_path: Option<String>,
    pub relic_reward_detection: Option<bool>,
    pub show_ocr_bounding_boxes: Option<bool>,
}

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> Result<SettingsPayload, String> {
    let app_state = app.state::<AppState>();
    let warframe_log_path = app_state
        .warframe_log_path
        .lock()
        .map_err(|_| "failed to read warframe log path".to_string())?
        .clone();
    let relic_reward_detection = app_state
        .relic_reward_detection
        .lock()
        .map(|v| *v)
        .map_err(|_| "failed to read relic reward detection".to_string())?;
    let show_ocr_bounding_boxes = app_state
        .show_ocr_bounding_boxes
        .lock()
        .map(|v| *v)
        .map_err(|_| "failed to read show_ocr_bounding_boxes".to_string())?;

    Ok(SettingsPayload {
        ocr_theme: ocr::get_ocr_theme_settings(app.clone(), app.state::<AppState>())?,
        overlay_duration_secs: ocr::get_overlay_duration_secs(app.state::<AppState>())?,
        overlay_toggle_mode: ocr::get_overlay_toggle_mode(app.state::<AppState>())?,
        ocr_dictionary_mapping: ocr::get_ocr_dictionary_mapping_settings(app.state::<AppState>())?,
        warframe_log_path,
        relic_reward_detection,
        show_ocr_bounding_boxes,
    })
}

#[tauri::command]
pub fn patch_settings<R: Runtime>(
    app: AppHandle<R>,
    patch: SettingsPatchPayload,
) -> Result<SettingsPayload, String> {
    if let Some(theme) = patch.ocr_theme {
        ocr::set_ocr_theme(app.clone(), app.state::<AppState>(), theme)?;
    }

    if let Some(seconds) = patch.overlay_duration_secs {
        ocr::set_overlay_duration_secs(app.clone(), app.state::<AppState>(), seconds)?;
    }

    if let Some(enabled) = patch.overlay_toggle_mode {
        ocr::set_overlay_toggle_mode(app.clone(), app.state::<AppState>(), enabled)?;
    }

    if let Some(enabled) = patch.ocr_dictionary_mapping_enabled {
        ocr::set_ocr_dictionary_mapping_enabled(app.clone(), app.state::<AppState>(), enabled)?;
    }

    if let Some(threshold) = patch.ocr_dictionary_match_threshold {
        ocr::set_ocr_dictionary_match_threshold(app.clone(), app.state::<AppState>(), threshold)?;
    }

    if let Some(path) = patch.warframe_log_path {
        let normalized_path = validate_warframe_log_path(&path)?;

        let app_state = app.state::<AppState>();
        let mut warframe_log_path = app_state
            .warframe_log_path
            .lock()
            .map_err(|_| "failed to update warframe log path".to_string())?;
        *warframe_log_path = normalized_path.clone();

        let store = app
            .store(SETTINGS_STORE_PATH)
            .map_err(|err| format!("failed to open settings store: {err}"))?;
        store.set("warframe_log_path", normalized_path);
        let _ = store.save();
    }

    if let Some(enabled) = patch.relic_reward_detection {
        let app_state = app.state::<AppState>();
        let mut relic_reward_detection = app_state
            .relic_reward_detection
            .lock()
            .map_err(|_| "failed to update relic reward detection".to_string())?;
        *relic_reward_detection = enabled;

        let store = app
            .store(SETTINGS_STORE_PATH)
            .map_err(|err| format!("failed to open settings store: {err}"))?;
        store.set("relic_reward_detection", enabled);
        let _ = store.save();
    }

    if let Some(enabled) = patch.show_ocr_bounding_boxes {
        let app_state = app.state::<AppState>();
        let mut show_ocr_bounding_boxes = app_state
            .show_ocr_bounding_boxes
            .lock()
            .map_err(|_| "failed to update show_ocr_bounding_boxes".to_string())?;
        *show_ocr_bounding_boxes = enabled;

        let store = app
            .store(SETTINGS_STORE_PATH)
            .map_err(|err| format!("failed to open settings store: {err}"))?;
        store.set("show_ocr_bounding_boxes", enabled);
        let _ = store.save();
    }

    get_settings(app)
}

fn validate_warframe_log_path(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let path = Path::new(trimmed);
    if !path.exists() || !path.is_file() {
        return Err("warframe log path must point to an existing file".to_string());
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(format!(
            "warframe log path must point to a file named {WARFRAME_LOG_FILENAME}"
        ));
    };

    if file_name != WARFRAME_LOG_FILENAME {
        return Err(format!(
            "warframe log file must be named {WARFRAME_LOG_FILENAME}"
        ));
    }

    Ok(trimmed.to_string())
}
