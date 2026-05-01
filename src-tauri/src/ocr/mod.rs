pub mod capture;
pub mod dictionary;
pub mod engine;
pub mod preprocessing;

pub use capture::*;
pub use dictionary::*;
pub use engine::*;
pub use preprocessing::*;

use serde::Serialize;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_store::StoreExt;

use crate::state::AppState;

pub const OCR_WHITELIST: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789[]- ";
pub const PASS_IMAGE_TO_FRONTEND: bool = true;
pub const PASS_TEXT_TO_FRONTEND: bool = false;
pub const DEFAULT_OCR_THEME: &str = "EQUINOX";
pub const DEFAULT_OCR_TARGET_RGB: [u8; 3] = [158, 159, 167];
pub const DEFAULT_OVERLAY_DURATION_SECS: u64 = 10;
pub const DEFAULT_OVERLAY_TOGGLE_MODE: bool = false;
pub const SETTINGS_STORE_PATH: &str = "settings.json";
pub const OVERLAY_DURATION_STORE_KEY: &str = "overlay_duration_secs";
pub const OVERLAY_TOGGLE_MODE_STORE_KEY: &str = "overlay_toggle_mode";
pub const OCR_THEME_STORE_KEY: &str = "ocr_theme";
pub const OCR_DICTIONARY_MAPPING_ENABLED_STORE_KEY: &str = "ocr_dictionary_mapping_enabled";
pub const OCR_DICTIONARY_MATCH_THRESHOLD_STORE_KEY: &str = "ocr_dictionary_match_threshold";
pub const THEME_COLORS_TOML: &str = include_str!("../theme_colors.toml");
pub const ENABLE_MORPHOLOGY: bool = false;
pub const ENABLE_OCR_DICTIONARY_MAPPING: bool = true;
pub const OCR_DICTIONARY_API_URL: &str = "http://api.thaumictom.de/warframe/v1/dictionary.json";
pub const TRADEABLE_ITEMS_API_URL: &str = "http://api.thaumictom.de/warframe/v1/tradeable_items.json";
pub const OCR_DICTIONARY_HTTP_TIMEOUT_SECS: u64 = 10;
pub const DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED: bool = true;
pub const DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 0.62;
pub const MIN_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 0.0;
pub const MAX_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 1.0;
pub const CUSTOM_OCR_DICTIONARY_ITEMS: [&str; 7] = [
    "Forma Blueprint",
    "2 X Forma Blueprint",
    "3 X Forma Blueprint",
    "Riven Sliver",
    "1,200 X Kuva",
    "Ayatan Amber Star",
    "Exilus Weapon Adapter Blueprint",
];
#[cfg(target_os = "windows")]
pub const EMBEDDED_TRAINEDDATA_BYTES: &[u8] = include_bytes!(env!("OCR_EMBEDDED_TRAINEDDATA_PATH"));
#[cfg(target_os = "windows")]
pub const EMBEDDED_TRAINEDDATA_FILENAME: &str = env!("OCR_EMBEDDED_TRAINEDDATA_FILENAME");
pub const BINARY_FILTER_SPILL_THRESHOLD: u8 = 0;
pub const HORIZONTAL_WORD_GAP_FACTOR: f64 = 0.75;
pub const SAME_LINE_VERTICAL_FACTOR: f64 = 0.25;
pub const MERGE_LINE_VERTICAL_FACTOR: f64 = 1.0;
pub const MAX_MERGED_LINES: usize = 3;
pub const CENTER_ALIGNED_MERGE_FACTOR: f64 = 3.0;
pub const CENTER_ALIGNED_HORIZONTAL_GAP_FACTOR: f64 = 3.0;

#[derive(Debug, Clone, Serialize)]
pub struct OcrWord {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_median: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_median_from_current_offers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ducats: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaulted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_custom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades_24h: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moving_avg: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrPayload {
    pub words: Vec<OcrWord>,
    pub show_ocr_bounding_boxes: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrDebugImagePayload {
    pub png_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub upscale_amount: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrTextPayload {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrThemeOption {
    pub name: String,
    pub rgb: [u8; 3],
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrThemeSettingsPayload {
    pub themes: Vec<OcrThemeOption>,
    pub selected_theme: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrDictionaryMappingSettingsPayload {
    pub enabled: bool,
    pub threshold: f64,
    pub hard_disabled: bool,
    pub min_threshold: f64,
    pub max_threshold: f64,
}

#[tauri::command]
pub fn get_ocr_theme_settings<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<OcrThemeSettingsPayload, String> {
    let themes = load_primary_theme_options(&app)?;
    if themes.is_empty() {
        return Err("no primary OCR themes found in theme_colors.toml".to_string());
    }

    let fallback = themes
        .iter()
        .find(|theme| theme.name == DEFAULT_OCR_THEME)
        .cloned()
        .unwrap_or_else(|| themes[0].clone());

    let mut selected_theme = state
        .ocr_theme
        .lock()
        .map_err(|_| "failed to read OCR theme".to_string())?;

    let resolved = themes
        .iter()
        .find(|theme| theme.name == *selected_theme)
        .cloned()
        .unwrap_or_else(|| {
            *selected_theme = fallback.name.clone();
            fallback
        });

    let mut target_rgb = state
        .ocr_target_rgb
        .lock()
        .map_err(|_| "failed to read OCR target RGB".to_string())?;
    *target_rgb = resolved.rgb;

    Ok(OcrThemeSettingsPayload {
        themes,
        selected_theme: selected_theme.clone(),
    })
}

#[tauri::command]
pub fn set_ocr_theme<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    theme: String,
) -> Result<(), String> {
    let requested_theme = theme.trim();
    if requested_theme.is_empty() {
        return Err("theme must not be empty".to_string());
    }

    apply_ocr_theme(&app, requested_theme)?;

    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;

    let current_theme = state
        .ocr_theme
        .lock()
        .map_err(|_| "failed to read applied OCR theme".to_string())?
        .clone();

    store.set(OCR_THEME_STORE_KEY, current_theme);
    store
        .save()
        .map_err(|err| format!("failed to save OCR theme: {err}"))?;

    Ok(())
}

#[tauri::command]
pub fn get_overlay_duration_secs(state: State<'_, AppState>) -> Result<u64, String> {
    state
        .overlay_duration_secs
        .lock()
        .map(|value| *value)
        .map_err(|_| "failed to read overlay duration".to_string())
}

#[tauri::command]
pub fn set_overlay_duration_secs<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    seconds: u64,
) -> Result<u64, String> {
    if seconds == 0 {
        return Err("overlay duration must be a positive number of seconds".to_string());
    }

    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;
    store.set(OVERLAY_DURATION_STORE_KEY, seconds);
    store
        .save()
        .map_err(|err| format!("failed to save overlay duration: {err}"))?;

    let mut overlay_duration_secs = state
        .overlay_duration_secs
        .lock()
        .map_err(|_| "failed to update overlay duration".to_string())?;
    *overlay_duration_secs = seconds;
    Ok(*overlay_duration_secs)
}

#[tauri::command]
pub fn get_overlay_toggle_mode(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .overlay_toggle_mode
        .lock()
        .map(|value| *value)
        .map_err(|_| "failed to read overlay mode".to_string())
}

#[tauri::command]
pub fn set_overlay_toggle_mode<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<bool, String> {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;
    store.set(OVERLAY_TOGGLE_MODE_STORE_KEY, enabled);
    store
        .save()
        .map_err(|err| format!("failed to save overlay mode: {err}"))?;

    let mut overlay_toggle_mode = state
        .overlay_toggle_mode
        .lock()
        .map_err(|_| "failed to update overlay mode".to_string())?;
    *overlay_toggle_mode = enabled;
    Ok(*overlay_toggle_mode)
}

#[tauri::command]
pub fn get_ocr_dictionary_mapping_settings(
    state: State<'_, AppState>,
) -> Result<OcrDictionaryMappingSettingsPayload, String> {
    let enabled = state
        .ocr_dictionary_mapping_enabled
        .lock()
        .map(|value| *value)
        .map_err(|_| "failed to read OCR dictionary mapping toggle".to_string())?;

    let threshold = state
        .ocr_dictionary_match_threshold
        .lock()
        .map(|value| *value)
        .map_err(|_| "failed to read OCR dictionary threshold".to_string())?
        .clamp(
            MIN_OCR_DICTIONARY_MATCH_THRESHOLD,
            MAX_OCR_DICTIONARY_MATCH_THRESHOLD,
        );

    Ok(OcrDictionaryMappingSettingsPayload {
        enabled,
        threshold,
        hard_disabled: !ENABLE_OCR_DICTIONARY_MAPPING,
        min_threshold: MIN_OCR_DICTIONARY_MATCH_THRESHOLD,
        max_threshold: MAX_OCR_DICTIONARY_MATCH_THRESHOLD,
    })
}

#[tauri::command]
pub fn set_ocr_dictionary_mapping_enabled<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<bool, String> {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;
    store.set(OCR_DICTIONARY_MAPPING_ENABLED_STORE_KEY, enabled);
    store
        .save()
        .map_err(|err| format!("failed to save OCR dictionary mapping toggle: {err}"))?;

    let mut mapping_enabled = state
        .ocr_dictionary_mapping_enabled
        .lock()
        .map_err(|_| "failed to update OCR dictionary mapping toggle".to_string())?;
    *mapping_enabled = enabled;
    Ok(*mapping_enabled)
}

#[tauri::command]
pub fn set_ocr_dictionary_match_threshold<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    threshold: f64,
) -> Result<f64, String> {
    if !threshold.is_finite()
        || !(MIN_OCR_DICTIONARY_MATCH_THRESHOLD..=MAX_OCR_DICTIONARY_MATCH_THRESHOLD)
            .contains(&threshold)
    {
        return Err(format!(
            "OCR dictionary threshold must be between {MIN_OCR_DICTIONARY_MATCH_THRESHOLD} and {MAX_OCR_DICTIONARY_MATCH_THRESHOLD}"
        ));
    }

    let normalized_threshold = (threshold * 100.0).round() / 100.0;

    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;
    store.set(
        OCR_DICTIONARY_MATCH_THRESHOLD_STORE_KEY,
        normalized_threshold,
    );
    store
        .save()
        .map_err(|err| format!("failed to save OCR dictionary threshold: {err}"))?;

    let mut match_threshold = state
        .ocr_dictionary_match_threshold
        .lock()
        .map_err(|_| "failed to update OCR dictionary threshold".to_string())?;
    *match_threshold = normalized_threshold;
    Ok(*match_threshold)
}
