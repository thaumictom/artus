use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

use crate::ocr;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct SettingsPayload {
    pub ocr_theme: ocr::OcrThemeSettingsPayload,
    pub overlay_duration_secs: u64,
    pub overlay_toggle_mode: bool,
    pub ocr_dictionary_mapping: ocr::OcrDictionaryMappingSettingsPayload,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SettingsPatchPayload {
    pub ocr_theme: Option<String>,
    pub overlay_duration_secs: Option<u64>,
    pub overlay_toggle_mode: Option<bool>,
    pub ocr_dictionary_mapping_enabled: Option<bool>,
    pub ocr_dictionary_match_threshold: Option<f64>,
}

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> Result<SettingsPayload, String> {
    Ok(SettingsPayload {
        ocr_theme: ocr::get_ocr_theme_settings(app.clone(), app.state::<AppState>())?,
        overlay_duration_secs: ocr::get_overlay_duration_secs(app.state::<AppState>())?,
        overlay_toggle_mode: ocr::get_overlay_toggle_mode(app.state::<AppState>())?,
        ocr_dictionary_mapping: ocr::get_ocr_dictionary_mapping_settings(app.state::<AppState>())?,
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

    get_settings(app)
}
