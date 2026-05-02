//! OCR module — screen capture, text recognition, and dictionary matching.
//!
//! Sub-modules:
//! - [`capture`]:       Screen capture, Tesseract invocation, and overlay display.
//! - [`dictionary`]:    Remote dictionary fetching, fuzzy matching, and price lookups.
//! - [`engine`]:        Tessdata resolution, word grouping, and geometric helpers.
//! - [`preprocessing`]: Binary color filtering and morphological operations.

pub mod capture;
pub mod dictionary;
pub mod engine;
pub mod preprocessing;

// ── Public API (explicit re-exports) ──────────────────────────────────────────

pub use capture::{capture_active_window, capture_active_window_with_mode, toggle_overlay_hotkey};
pub use dictionary::{
    load_ocr_dictionary, load_primary_theme_options, load_tradeable_item_prices,
    map_words_to_dictionary,
};
pub use engine::{group_words_into_blocks, resolve_tessdata};
pub use preprocessing::{apply_morphology, binary_target_filter, gray_to_png_bytes};

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::error::AppResult;

// ── Feature flags ─────────────────────────────────────────────────────────────

pub const PASS_IMAGE_TO_FRONTEND: bool = true;
pub const PASS_TEXT_TO_FRONTEND: bool = false;
pub const ENABLE_MORPHOLOGY: bool = false;
pub const ENABLE_OCR_DICTIONARY_MAPPING: bool = true;

// ── OCR engine configuration ──────────────────────────────────────────────────

/// Characters Tesseract is allowed to output.
pub const OCR_WHITELIST: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789&[]- ";

// ── Preprocessing thresholds ──────────────────────────────────────────────────

/// Maximum per-channel difference allowed when matching a pixel to the target color.
pub const BINARY_FILTER_SPILL_THRESHOLD: u8 = 0;

// ── Word-grouping parameters ──────────────────────────────────────────────────

pub const HORIZONTAL_WORD_GAP_FACTOR: f64 = 0.75;
pub const SAME_LINE_VERTICAL_FACTOR: f64 = 0.25;
pub const MERGE_LINE_VERTICAL_FACTOR: f64 = 1.0;
pub const MAX_MERGED_LINES: usize = 3;
pub const CENTER_ALIGNED_MERGE_FACTOR: f64 = 3.0;
pub const CENTER_ALIGNED_HORIZONTAL_GAP_FACTOR: f64 = 3.0;

// ── Default values for user-configurable settings ─────────────────────────────

pub const DEFAULT_OCR_TARGET_RGB: [u8; 3] = [158, 159, 167];
pub const DEFAULT_OVERLAY_DURATION_SECS: u64 = 10;
pub const DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED: bool = true;
pub const DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 0.62;
pub const MIN_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 0.0;
pub const MAX_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 1.0;

// ── Embedded data ─────────────────────────────────────────────────────────────

pub const THEME_COLORS_TOML: &str = include_str!("../theme_colors.toml");

#[cfg(target_os = "windows")]
pub const EMBEDDED_TRAINEDDATA_BYTES: &[u8] = include_bytes!(env!("OCR_EMBEDDED_TRAINEDDATA_PATH"));
#[cfg(target_os = "windows")]
pub const EMBEDDED_TRAINEDDATA_FILENAME: &str = env!("OCR_EMBEDDED_TRAINEDDATA_FILENAME");

// ── API endpoints ─────────────────────────────────────────────────────────────

pub const OCR_DICTIONARY_API_URL: &str = "http://api.thaumictom.de/warframe/v1/dictionary.json";
pub const TRADEABLE_ITEMS_API_URL: &str =
    "http://api.thaumictom.de/warframe/v1/tradeable_items.json";
pub const OCR_DICTIONARY_HTTP_TIMEOUT_SECS: u64 = 10;

// ── Custom dictionary items (not in the remote API) ───────────────────────────

pub const CUSTOM_OCR_DICTIONARY_ITEMS: [&str; 7] = [
    "Forma Blueprint",
    "2 X Forma Blueprint",
    "3 X Forma Blueprint",
    "Riven Sliver",
    "1,200 X Kuva",
    "Ayatan Amber Star",
    "Exilus Weapon Adapter Blueprint",
];

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModType {
    Gold,
    Silver,
    Bronze,
    Archon,
    Special,
}

/// A single recognized word/block with optional dictionary match metadata.
#[derive(Debug, Clone, Serialize, Default)]
pub struct OcrWord {
    pub text: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
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
    pub relic_price_is_fallback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ducats: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaulted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_custom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_relic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades_24h: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moving_avg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_type: Option<ModType>,
}

impl OcrWord {
    /// Create a word with geometry only; all optional fields default to `None`.
    pub fn new(text: String, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            text,
            x,
            y,
            width,
            height,
            ..Default::default()
        }
    }
}

/// Payload emitted to the frontend with OCR results.
#[derive(Debug, Clone, Serialize)]
pub struct OcrPayload {
    pub words: Vec<OcrWord>,
    pub show_ocr_bounding_boxes: bool,
}

/// Debug image sent to the dashboard for visual inspection.
#[derive(Debug, Clone, Serialize)]
pub struct OcrDebugImagePayload {
    pub png_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub upscale_amount: u32,
}

/// Raw OCR text sent to the dashboard (when enabled).
#[derive(Debug, Clone, Serialize)]
pub struct OcrTextPayload {
    pub text: String,
}

/// A selectable theme entry shown in the settings UI.
#[derive(Debug, Clone, Serialize)]
pub struct OcrThemeOption {
    pub name: String,
    pub rgb: [u8; 3],
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Returns all available OCR theme options parsed from the embedded TOML.
#[tauri::command]
pub fn get_ocr_themes<R: Runtime>(app: AppHandle<R>) -> AppResult<Vec<OcrThemeOption>> {
    load_primary_theme_options(&app)
}
