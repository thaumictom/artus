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

#[tauri::command]
pub fn get_ocr_themes<R: Runtime>(app: AppHandle<R>) -> Result<Vec<OcrThemeOption>, String> {
    load_primary_theme_options(&app)
}
