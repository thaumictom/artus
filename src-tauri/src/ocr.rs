use std::collections::{BTreeMap, HashMap};
use std::env;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use image::{DynamicImage, GrayImage, ImageFormat};
use imageproc::distance_transform::Norm;
use imageproc::morphology::{dilate_mut, erode_mut};
use kreuzberg_tesseract::{TessPageIteratorLevel, TessPageSegMode, TesseractAPI};
use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Runtime, Size, State,
};
use tauri_plugin_store::StoreExt;
use xcap::Window;

use crate::layer_shell;
use crate::state::{AppState, OcrDictionaryEntry, TradeablePriceEntry};

const OCR_WHITELIST: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789[]- ";
const PASS_IMAGE_TO_FRONTEND: bool = true;
const PASS_TEXT_TO_FRONTEND: bool = false;
const DEFAULT_OCR_THEME: &str = "EQUINOX";
const DEFAULT_OCR_TARGET_RGB: [u8; 3] = [158, 159, 167];
const DEFAULT_OVERLAY_DURATION_SECS: u64 = 10;
const MIN_OVERLAY_DURATION_SECS: u64 = 1;
const MAX_OVERLAY_DURATION_SECS: u64 = 60;
const DEFAULT_OVERLAY_TOGGLE_MODE: bool = false;
const SETTINGS_STORE_PATH: &str = "settings.json";
const OVERLAY_DURATION_STORE_KEY: &str = "overlay_duration_secs";
const OVERLAY_TOGGLE_MODE_STORE_KEY: &str = "overlay_toggle_mode";
const OCR_DICTIONARY_MAPPING_ENABLED_STORE_KEY: &str = "ocr_dictionary_mapping_enabled";
const OCR_DICTIONARY_MATCH_THRESHOLD_STORE_KEY: &str = "ocr_dictionary_match_threshold";
const THEME_COLORS_TOML: &str = include_str!("theme_colors.toml");
const ENABLE_MORPHOLOGY: bool = false;
const ENABLE_OCR_DICTIONARY_MAPPING: bool = true;
const OCR_DICTIONARY_API_URL: &str = "http://api.thaumictom.de/warframe/v1/dictionary.json";
const TRADEABLE_ITEMS_API_URL: &str = "http://api.thaumictom.de/warframe/v1/tradeable_items.json";
const OCR_DICTIONARY_HTTP_TIMEOUT_SECS: u64 = 10;
const DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED: bool = true;
const DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 0.62;
const MIN_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 0.0;
const MAX_OCR_DICTIONARY_MATCH_THRESHOLD: f64 = 1.0;
#[cfg(target_os = "windows")]
const EMBEDDED_TRAINEDDATA_BYTES: &[u8] = include_bytes!(env!("OCR_EMBEDDED_TRAINEDDATA_PATH"));
#[cfg(target_os = "windows")]
const EMBEDDED_TRAINEDDATA_FILENAME: &str = env!("OCR_EMBEDDED_TRAINEDDATA_FILENAME");
// Allowed per-channel RGB distance from the target color for a pixel to be treated as text.
pub const BINARY_FILTER_SPILL_THRESHOLD: u8 = 0;
// Max horizontal gap (scaled by average word height) for joining words on the same line.
pub const HORIZONTAL_WORD_GAP_FACTOR: f64 = 0.75;
// Max vertical center distance (scaled by line height) for assigning words to one line.
pub const SAME_LINE_VERTICAL_FACTOR: f64 = 0.25;
// Max vertical gap (scaled by line height) for merging nearby lines into one block.
pub const MERGE_LINE_VERTICAL_FACTOR: f64 = 1.0;
// Maximum number of detected lines to merge into a single OCR block.
pub const MAX_MERGED_LINES: usize = 3;
// Max horizontal center offset (scaled by line height) tolerated for center-aligned line merges.
pub const CENTER_ALIGNED_MERGE_FACTOR: f64 = 3.0;
// Max horizontal box gap (scaled by line height) allowed when center-aligned lines do not overlap.
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
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrPayload {
    pub words: Vec<OcrWord>,
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

#[derive(Debug, Deserialize)]
struct ThemeColorsToml {
    #[serde(default)]
    primary: BTreeMap<String, [u8; 3]>,
}

#[derive(Debug, Deserialize)]
struct DictionaryApiResponse {
    #[serde(default)]
    tradeable_items: Vec<DictionaryApiItem>,
}

#[derive(Debug, Deserialize)]
struct DictionaryApiItem {
    name: String,
    slug: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemsApiResponse {
    #[serde(default)]
    tradeable_items: Vec<TradeableItemApiItem>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemApiItem {
    slug: String,
    #[serde(default)]
    statistics_today: Vec<TradeableItemStats>,
    #[serde(default)]
    current_offers: Vec<TradeableItemStats>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemStats {
    median: Option<f64>,
}

#[derive(Debug, Clone)]
struct RawWord {
    text: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
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

    let themes = load_primary_theme_options(&app)?;
    let selected = themes
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(requested_theme))
        .cloned()
        .ok_or_else(|| format!("unknown OCR theme: {requested_theme}"))?;

    let mut selected_theme = state
        .ocr_theme
        .lock()
        .map_err(|_| "failed to update OCR theme".to_string())?;
    let mut target_rgb = state
        .ocr_target_rgb
        .lock()
        .map_err(|_| "failed to update OCR target RGB".to_string())?;

    *selected_theme = selected.name;
    *target_rgb = selected.rgb;
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
    if !(MIN_OVERLAY_DURATION_SECS..=MAX_OVERLAY_DURATION_SECS).contains(&seconds) {
        return Err(format!(
            "overlay duration must be between {MIN_OVERLAY_DURATION_SECS} and {MAX_OVERLAY_DURATION_SECS} seconds"
        ));
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

pub fn load_persisted_overlay_duration<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to load settings store: {err}"))?;

    let Some(saved_value) = store.get(OVERLAY_DURATION_STORE_KEY) else {
        return Ok(());
    };

    let Some(saved_seconds) = saved_value.as_u64() else {
        return Ok(());
    };

    if !(MIN_OVERLAY_DURATION_SECS..=MAX_OVERLAY_DURATION_SECS).contains(&saved_seconds) {
        return Ok(());
    }

    let app_state = app.state::<AppState>();
    let mut overlay_duration_secs = app_state
        .overlay_duration_secs
        .lock()
        .map_err(|_| "failed to apply persisted overlay duration".to_string())?;
    *overlay_duration_secs = saved_seconds;
    Ok(())
}

pub fn load_persisted_overlay_mode<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to load settings store: {err}"))?;

    let Some(saved_value) = store.get(OVERLAY_TOGGLE_MODE_STORE_KEY) else {
        let app_state = app.state::<AppState>();
        let mut overlay_toggle_mode = app_state
            .overlay_toggle_mode
            .lock()
            .map_err(|_| "failed to apply default overlay mode".to_string())?;
        *overlay_toggle_mode = DEFAULT_OVERLAY_TOGGLE_MODE;
        return Ok(());
    };

    let Some(saved_mode) = saved_value.as_bool() else {
        return Ok(());
    };

    let app_state = app.state::<AppState>();
    let mut overlay_toggle_mode = app_state
        .overlay_toggle_mode
        .lock()
        .map_err(|_| "failed to apply persisted overlay mode".to_string())?;
    *overlay_toggle_mode = saved_mode;
    Ok(())
}

pub fn load_persisted_ocr_dictionary_mapping_settings<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to load settings store: {err}"))?;

    let app_state = app.state::<AppState>();

    let saved_mapping_enabled = store
        .get(OCR_DICTIONARY_MAPPING_ENABLED_STORE_KEY)
        .and_then(|value| value.as_bool())
        .unwrap_or(DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED);

    let saved_threshold = store
        .get(OCR_DICTIONARY_MATCH_THRESHOLD_STORE_KEY)
        .and_then(|value| value.as_f64())
        .filter(|value| {
            value.is_finite()
                && (MIN_OCR_DICTIONARY_MATCH_THRESHOLD..=MAX_OCR_DICTIONARY_MATCH_THRESHOLD)
                    .contains(value)
        })
        .unwrap_or(DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD);

    let mut mapping_enabled = app_state
        .ocr_dictionary_mapping_enabled
        .lock()
        .map_err(|_| "failed to apply OCR dictionary mapping toggle".to_string())?;
    *mapping_enabled = saved_mapping_enabled;
    drop(mapping_enabled);

    let mut match_threshold = app_state
        .ocr_dictionary_match_threshold
        .lock()
        .map_err(|_| "failed to apply OCR dictionary threshold".to_string())?;
    *match_threshold = saved_threshold;

    Ok(())
}

pub fn load_ocr_dictionary<R: Runtime>(app: &AppHandle<R>) -> Result<usize, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(OCR_DICTIONARY_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("failed to build dictionary client: {err}"))?;

    let response = client
        .get(OCR_DICTIONARY_API_URL)
        .send()
        .map_err(|err| format!("failed to fetch dictionary: {err}"))?
        .error_for_status()
        .map_err(|err| format!("dictionary request failed: {err}"))?;

    let payload: DictionaryApiResponse = response
        .json()
        .map_err(|err| format!("failed to parse dictionary response: {err}"))?;

    let mut dictionary_entries = payload
        .tradeable_items
        .into_iter()
        .filter_map(|item| {
            let name = item.name.trim();
            let slug = item.slug.trim();
            if name.is_empty() || slug.is_empty() {
                return None;
            }

            let normalized_name = normalize_dictionary_text(name);
            if normalized_name.is_empty() {
                return None;
            }

            Some(OcrDictionaryEntry {
                name: name.to_string(),
                slug: slug.to_string(),
                tags: item.tags,
                normalized_name,
            })
        })
        .collect::<Vec<_>>();

    dictionary_entries.sort_by(|left, right| left.normalized_name.cmp(&right.normalized_name));
    dictionary_entries.dedup_by(|left, right| left.normalized_name == right.normalized_name);

    let dictionary_len = dictionary_entries.len();
    let app_state = app.state::<AppState>();
    let mut dictionary = app_state
        .ocr_dictionary
        .lock()
        .map_err(|_| "failed to store OCR dictionary".to_string())?;
    *dictionary = dictionary_entries;

    Ok(dictionary_len)
}

pub fn load_tradeable_item_prices<R: Runtime>(app: &AppHandle<R>) -> Result<usize, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(OCR_DICTIONARY_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("failed to build tradeable item client: {err}"))?;

    let response = client
        .get(TRADEABLE_ITEMS_API_URL)
        .send()
        .map_err(|err| format!("failed to fetch tradeable items: {err}"))?
        .error_for_status()
        .map_err(|err| format!("tradeable items request failed: {err}"))?;

    let payload: TradeableItemsApiResponse = response
        .json()
        .map_err(|err| format!("failed to parse tradeable items response: {err}"))?;

    let mut prices_by_slug = HashMap::new();

    for item in payload.tradeable_items {
        let slug = item.slug.trim();
        if slug.is_empty() {
            continue;
        }

        let statistics_today_median = item
            .statistics_today
            .first()
            .and_then(|entry| entry.median)
            .filter(|median| median.is_finite());

        let current_offers_median = item
            .current_offers
            .first()
            .and_then(|entry| entry.median)
            .filter(|median| median.is_finite());

        let Some((median, used_fallback)) = statistics_today_median
            .map(|value| (value, false))
            .or_else(|| current_offers_median.map(|value| (value, true)))
        else {
            continue;
        };

        prices_by_slug.insert(
            slug.to_string(),
            TradeablePriceEntry {
                median,
                used_current_offer_fallback: used_fallback,
            },
        );
    }

    let count = prices_by_slug.len();
    let app_state = app.state::<AppState>();
    let mut prices_guard = app_state
        .ocr_tradeable_prices
        .lock()
        .map_err(|_| "failed to store tradeable prices".to_string())?;
    *prices_guard = prices_by_slug;

    Ok(count)
}

pub fn capture_active_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    capture_active_window_with_mode(app, true)
}

pub fn toggle_overlay_hotkey<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("overlay window not found")?;
    let is_visible = overlay
        .is_visible()
        .map_err(|err| format!("failed to read overlay visibility: {err}"))?;

    if is_visible {
        let _ = bump_overlay_sequence(app)?;
        overlay
            .hide()
            .map_err(|err| format!("failed to hide overlay: {err}"))?;
        return Ok(());
    }

    capture_active_window_with_mode(app, false)
}

fn bump_overlay_sequence<R: Runtime>(app: &AppHandle<R>) -> Result<u64, String> {
    let app_state = app.state::<AppState>();
    let mut guard = app_state
        .overlay_sequence
        .lock()
        .map_err(|_| "failed to update overlay sequence".to_string())?;
    *guard += 1;
    Ok(*guard)
}

fn capture_active_window_with_mode<R: Runtime>(
    app: &AppHandle<R>,
    should_auto_hide: bool,
) -> Result<(), String> {
    let total_started = Instant::now();

    let discovery_started = Instant::now();
    let window = Window::all()
        .map_err(|err| format!("failed to list windows: {err}"))?
        .into_iter()
        .find(|entry| entry.is_focused().unwrap_or(false) && !entry.is_minimized().unwrap_or(false))
        .ok_or("no focused window found")?;

    let x = window
        .x()
        .map_err(|err| format!("failed to get x: {err}"))?;
    let y = window
        .y()
        .map_err(|err| format!("failed to get y: {err}"))?;
    let width = window
        .width()
        .map_err(|err| format!("failed to get width: {err}"))?;
    let height = window
        .height()
        .map_err(|err| format!("failed to get height: {err}"))?;

    let image = window
        .capture_image()
        .map_err(|err| format!("failed to capture active window: {err}"))?;
    println!(
        "[ocr] window discovery + capture: {:?}",
        discovery_started.elapsed()
    );

    let preprocess_started = Instant::now();
    let target_rgb = app
        .state::<AppState>()
        .ocr_target_rgb
        .lock()
        .map(|value| *value)
        .unwrap_or(DEFAULT_OCR_TARGET_RGB);

    let mut filtered = binary_target_filter(&image, target_rgb);
    apply_morphology(&mut filtered);

    println!(
        "[ocr] preprocess (binary filter + erosion): {:?}",
        preprocess_started.elapsed()
    );

    if PASS_IMAGE_TO_FRONTEND {
        let debug_started = Instant::now();
        let png_bytes = gray_to_png_bytes(&filtered)?;
        if let Some(dashboard) = app.get_webview_window("artus") {
            let _ = dashboard.emit(
                "ocr_debug_image",
                OcrDebugImagePayload {
                    png_bytes,
                    width: filtered.width(),
                    height: filtered.height(),
                    upscale_amount: 1,
                },
            );
        }
        println!(
            "[ocr] debug image encode + emit: {:?}",
            debug_started.elapsed()
        );
    }

    let ocr_started = Instant::now();
    let tessdata = resolve_tessdata(app)?;
    let api = TesseractAPI::new();
    api.init(&tessdata, "eng")
        .map_err(|err| format!("failed to init tesseract: {err}"))?;

    api.set_page_seg_mode(TessPageSegMode::PSM_SPARSE_TEXT)
        .map_err(|err| format!("failed to set page segmentation: {err}"))?;
    api.set_variable("tessedit_char_whitelist", OCR_WHITELIST)
        .map_err(|err| format!("failed to set whitelist: {err}"))?;
    api.set_image(
        filtered.as_raw(),
        filtered.width() as i32,
        filtered.height() as i32,
        1,
        filtered.width() as i32,
    )
    .map_err(|err| format!("failed to set image: {err}"))?;

    api.recognize()
        .map_err(|err| format!("recognition failed: {err}"))?;
    let iter = api
        .get_iterator()
        .map_err(|err| format!("failed to get OCR iterator: {err}"))?;
    println!(
        "[ocr] tesseract setup + recognize: {:?}",
        ocr_started.elapsed()
    );

    let parse_started = Instant::now();
    let mut words = Vec::new();
    loop {
        let text = iter
            .get_utf8_text(TessPageIteratorLevel::RIL_WORD)
            .unwrap_or_default()
            .trim()
            .chars()
            .filter(|ch| OCR_WHITELIST.contains(*ch))
            .collect::<String>();

        if !text.is_empty() {
            if let Ok((left, top, right, bottom)) =
                iter.get_bounding_box(TessPageIteratorLevel::RIL_WORD)
            {
                words.push(OcrWord {
                    text,
                    x: left as f64,
                    y: top as f64,
                    width: (right - left) as f64,
                    height: (bottom - top) as f64,
                    slug: None,
                    mapping_confidence: None,
                    market_median: None,
                    market_median_from_current_offers: None,
                });
            }
        }

        let has_next = iter
            .next(TessPageIteratorLevel::RIL_WORD)
            .map_err(|err| format!("iterator error: {err}"))?;
        if !has_next {
            break;
        }
    }

    let app_state = app.state::<AppState>();
    let mapping_enabled = app_state
        .ocr_dictionary_mapping_enabled
        .lock()
        .map(|value| *value)
        .unwrap_or(DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED);
    let mapping_threshold = app_state
        .ocr_dictionary_match_threshold
        .lock()
        .map(|value| *value)
        .unwrap_or(DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD)
        .clamp(
            MIN_OCR_DICTIONARY_MATCH_THRESHOLD,
            MAX_OCR_DICTIONARY_MATCH_THRESHOLD,
        );

    let grouped_words = group_words_into_blocks(&words);
    let finalized_words = if ENABLE_OCR_DICTIONARY_MAPPING && mapping_enabled {
        map_words_to_dictionary(app, &grouped_words, mapping_threshold)
    } else {
        grouped_words.clone()
    };
    let mapped_count = finalized_words
        .iter()
        .filter(|word| word.slug.is_some())
        .count();
    let dropped_count = grouped_words.len().saturating_sub(finalized_words.len());
    println!(
        "[ocr] parse OCR words: {:?} ({} words -> {} blocks, {} mapped, {} dropped, mapping {}, threshold {:.2})",
        parse_started.elapsed(),
        words.len(),
        grouped_words.len(),
        mapped_count,
        dropped_count,
        ENABLE_OCR_DICTIONARY_MAPPING && mapping_enabled,
        mapping_threshold,
    );

    if PASS_TEXT_TO_FRONTEND {
        let text_started = Instant::now();
        let text = finalized_words
            .iter()
            .map(|word| word.text.trim())
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");

        if let Some(dashboard) = app.get_webview_window("artus") {
            let _ = dashboard.emit("ocr_text_result", OcrTextPayload { text });
        }
        println!("[ocr] text emit: {:?}", text_started.elapsed());
    }

    let overlay_started = Instant::now();
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("overlay window not found")?;

    let used_layer_shell_positioning =
        layer_shell::set_overlay_geometry(&overlay, x, y).unwrap_or(false);

    if !used_layer_shell_positioning {
        overlay
            .set_position(Position::Physical(PhysicalPosition::new(x, y)))
            .map_err(|err| format!("failed to position overlay: {err}"))?;
    }

    overlay
        .set_size(Size::Physical(PhysicalSize::new(width, height)))
        .map_err(|err| format!("failed to resize overlay: {err}"))?;
    overlay
        .show()
        .map_err(|err| format!("failed to show overlay: {err}"))?;

    if !layer_shell::is_wayland_session() || used_layer_shell_positioning {
        let _ = overlay.set_ignore_cursor_events(true);
        let _ = overlay.set_focusable(false);
    }

    let force_click_applied = layer_shell::force_click_through(&overlay).unwrap_or(false);
    if layer_shell::is_wayland_session() && !force_click_applied {
        eprintln!("[overlay] click-through not applied on initial show");
    }

    if layer_shell::is_wayland_session() {
        let app_handle_retry = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            if let Some(overlay_retry) = app_handle_retry.get_webview_window("overlay") {
                let _ = overlay_retry.set_ignore_cursor_events(true);
                let _ = overlay_retry.set_focusable(false);
                let retry_applied =
                    layer_shell::force_click_through(&overlay_retry).unwrap_or(false);
                if !retry_applied {
                    eprintln!("[overlay] click-through not applied on delayed retry");
                }
            }
        });
    }

    app.emit(
        "ocr_result",
        OcrPayload {
            words: finalized_words,
        },
    )
    .map_err(|err| format!("failed to emit OCR result: {err}"))?;
    println!("[ocr] overlay show + emit: {:?}", overlay_started.elapsed());

    let sequence = bump_overlay_sequence(app)?;

    if should_auto_hide {
        let app_handle = app.clone();
        let overlay_duration_secs = app
            .state::<AppState>()
            .overlay_duration_secs
            .lock()
            .map(|value| *value)
            .unwrap_or(DEFAULT_OVERLAY_DURATION_SECS);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(overlay_duration_secs));
            let current_sequence = app_handle
                .state::<AppState>()
                .overlay_sequence
                .lock()
                .map(|value| *value)
                .unwrap_or(0);
            if current_sequence != sequence {
                return;
            }

            if let Some(overlay) = app_handle.get_webview_window("overlay") {
                let _ = overlay.hide();
            }
        });
    }

    println!("[ocr] total: {:?}", total_started.elapsed());

    Ok(())
}

fn binary_target_filter(source: &image::RgbaImage, target_rgb: [u8; 3]) -> GrayImage {
    let width = source.width() as usize;
    let height = source.height() as usize;
    let raw = source.as_raw();
    let mut output = vec![255u8; width * height];

    for y in 0..height {
        for x in 0..width {
            let src_idx = (y * width + x) * 4;
            let r = raw[src_idx];
            let g = raw[src_idx + 1];
            let b = raw[src_idx + 2];
            output[y * width + x] =
                if matches_target_color(r, g, b, target_rgb[0], target_rgb[1], target_rgb[2])
                // || matches_target_color(r, g, b, TARGET_R_ALT, TARGET_G_ALT, TARGET_B_ALT)
                {
                    0
                } else {
                    255
                };
        }
    }

    GrayImage::from_raw(source.width(), source.height(), output)
        .expect("invalid binary filter output dimensions")
}

fn apply_morphology(source: &mut GrayImage) {
    if !ENABLE_MORPHOLOGY {
        return;
    }

    // Experimental
    erode_mut(source, Norm::L1, 1);
    dilate_mut(source, Norm::L1, 1);
}

fn matches_target_color(r: u8, g: u8, b: u8, target_r: u8, target_g: u8, target_b: u8) -> bool {
    r.abs_diff(target_r) <= BINARY_FILTER_SPILL_THRESHOLD
        && g.abs_diff(target_g) <= BINARY_FILTER_SPILL_THRESHOLD
        && b.abs_diff(target_b) <= BINARY_FILTER_SPILL_THRESHOLD
}

fn gray_to_png_bytes(gray: &GrayImage) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    DynamicImage::ImageLuma8(gray.clone())
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| format!("failed to encode debug image: {err}"))?;
    Ok(bytes)
}

fn resolve_tessdata<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let mut checked_paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        match resolve_embedded_tessdata(app) {
            Ok(path) if has_traineddata_files(&path) => return Ok(path),
            Ok(path) => checked_paths.push(path.display().to_string()),
            Err(err) => checked_paths.push(format!("embedded: {err}")),
        }
    }

    let mut candidates = vec![];

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("tessdata"));
        candidates.push(resource_dir);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            candidates.push(exe_dir.join("tessdata"));
            candidates.push(exe_dir.to_path_buf());
        }
    }

    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("src-tauri").join("tessdata"));
        candidates.push(cwd.join("tessdata"));
    }

    for candidate in candidates {
        if has_traineddata_files(&candidate) {
            return Ok(candidate);
        }
        checked_paths.push(candidate.display().to_string());
    }

    Err(format!(
        "could not find tessdata directory (checked: {})",
        checked_paths.join(", ")
    ))
}

#[cfg(target_os = "windows")]
fn resolve_embedded_tessdata<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?;
    let tessdata_dir = app_data_dir.join("tessdata");

    std::fs::create_dir_all(&tessdata_dir)
        .map_err(|err| format!("failed to create embedded tessdata dir: {err}"))?;

    let traineddata_path = tessdata_dir.join(EMBEDDED_TRAINEDDATA_FILENAME);
    let should_write = std::fs::metadata(&traineddata_path)
        .map(|meta| meta.len() != EMBEDDED_TRAINEDDATA_BYTES.len() as u64)
        .unwrap_or(true);

    if should_write {
        std::fs::write(&traineddata_path, EMBEDDED_TRAINEDDATA_BYTES)
            .map_err(|err| format!("failed to write embedded traineddata: {err}"))?;
    }

    Ok(tessdata_dir)
}

fn has_traineddata_files(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("traineddata"))
            })
        })
        .unwrap_or(false)
}

fn load_primary_theme_options<R: Runtime>(
    _app: &AppHandle<R>,
) -> Result<Vec<OcrThemeOption>, String> {
    let parsed: ThemeColorsToml = toml::from_str(THEME_COLORS_TOML)
        .map_err(|err| format!("failed to parse embedded theme colors: {err}"))?;

    let themes = parsed
        .primary
        .into_iter()
        .map(|(name, rgb)| OcrThemeOption { name, rgb })
        .collect::<Vec<_>>();

    if themes.is_empty() {
        return Err("primary section in theme_colors.toml is empty".to_string());
    }

    Ok(themes)
}

fn map_words_to_dictionary<R: Runtime>(
    app: &AppHandle<R>,
    words: &[OcrWord],
    threshold: f64,
) -> Vec<OcrWord> {
    if words.is_empty() {
        return vec![];
    }

    let app_state = app.state::<AppState>();
    let dictionary_guard = match app_state.ocr_dictionary.lock() {
        Ok(guard) => guard,
        Err(_) => return words.to_vec(),
    };

    if dictionary_guard.is_empty() {
        return words.to_vec();
    }

    let needs_price_reload = app_state
        .ocr_tradeable_prices
        .lock()
        .map(|prices| prices.is_empty())
        .unwrap_or(false);

    if needs_price_reload {
        match load_tradeable_item_prices(app) {
            Ok(count) => println!("[ocr] lazy-loaded tradeable item prices: {count}"),
            Err(err) => eprintln!("[ocr] failed to lazy-load tradeable item prices: {err}"),
        }
    }

    let prices_guard = app_state.ocr_tradeable_prices.lock().ok();
    let prices_by_slug = prices_guard.as_deref();

    words
        .iter()
        .filter_map(|word| {
            map_word_to_dictionary(word, &dictionary_guard, prices_by_slug, threshold)
        })
        .collect()
}

fn map_word_to_dictionary(
    word: &OcrWord,
    dictionary: &[OcrDictionaryEntry],
    prices_by_slug: Option<&HashMap<String, TradeablePriceEntry>>,
    threshold: f64,
) -> Option<OcrWord> {
    let normalized_word = normalize_dictionary_text(&word.text);
    if normalized_word.is_empty() {
        return None;
    }
    let normalized_tokens = normalized_word.split_whitespace().collect::<Vec<_>>();

    let mut best_match: Option<(&OcrDictionaryEntry, f64)> = None;

    for candidate in dictionary {
        let tag_bonus = candidate
            .tags
            .iter()
            .filter_map(|tag| {
                let normalized_tag = normalize_dictionary_text(tag);
                if normalized_tag.is_empty() {
                    None
                } else {
                    Some(normalized_tag)
                }
            })
            .filter(|normalized_tag| {
                normalized_tokens
                    .iter()
                    .any(|token| *token == normalized_tag.as_str())
            })
            .count() as f64
            * 0.02;

        let score = (dictionary_similarity_score(&normalized_word, &candidate.normalized_name)
            + tag_bonus)
            .min(1.0);
        match best_match {
            Some((_, best_score)) if score <= best_score => {}
            _ => best_match = Some((candidate, score)),
        }
    }

    match best_match {
        Some((candidate, score)) if score >= threshold => {
            let mut mapped = word.clone();
            mapped.text = candidate.name.clone();
            mapped.slug = Some(candidate.slug.clone());
            mapped.mapping_confidence = Some(score);

            if let Some(prices_lookup) = prices_by_slug {
                if let Some(price_entry) = prices_lookup.get(&candidate.slug) {
                    mapped.market_median = Some(price_entry.median);
                    mapped.market_median_from_current_offers =
                        Some(price_entry.used_current_offer_fallback);
                }
            }

            Some(mapped)
        }
        _ => None,
    }
}

fn dictionary_similarity_score(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }

    let max_len = left.len().max(right.len());
    if max_len == 0 {
        return 0.0;
    }

    let distance = levenshtein_distance(left.as_bytes(), right.as_bytes());
    let levenshtein_score = 1.0 - distance as f64 / max_len as f64;
    let overlap_score = token_overlap_score(left, right);
    (levenshtein_score * 0.85 + overlap_score * 0.15).clamp(0.0, 1.0)
}

fn token_overlap_score(left: &str, right: &str) -> f64 {
    let left_tokens = left.split_whitespace().collect::<Vec<_>>();
    let right_tokens = right.split_whitespace().collect::<Vec<_>>();
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return 0.0;
    }

    let shared_count = left_tokens
        .iter()
        .filter(|token| right_tokens.contains(token))
        .count();
    shared_count as f64 / left_tokens.len().max(right_tokens.len()) as f64
}

fn normalize_dictionary_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn levenshtein_distance(left: &[u8], right: &[u8]) -> usize {
    if left.is_empty() {
        return right.len();
    }
    if right.is_empty() {
        return left.len();
    }

    let mut previous_row: Vec<usize> = (0..=right.len()).collect();
    let mut current_row = vec![0usize; right.len() + 1];

    for (left_index, left_byte) in left.iter().enumerate() {
        current_row[0] = left_index + 1;

        for (right_index, right_byte) in right.iter().enumerate() {
            let substitution_cost = if left_byte == right_byte { 0 } else { 1 };
            let delete_cost = previous_row[right_index + 1] + 1;
            let insert_cost = current_row[right_index] + 1;
            let substitute_cost = previous_row[right_index] + substitution_cost;

            current_row[right_index + 1] = delete_cost.min(insert_cost).min(substitute_cost);
        }

        std::mem::swap(&mut previous_row, &mut current_row);
    }

    previous_row[right.len()]
}

fn ranges_overlap(left_start: f64, left_end: f64, right_start: f64, right_end: f64) -> bool {
    left_start.max(right_start) <= left_end.min(right_end)
}

fn horizontal_gap(left_start: f64, left_end: f64, right_start: f64, right_end: f64) -> f64 {
    if left_end < right_start {
        right_start - left_end
    } else if right_end < left_start {
        left_start - right_end
    } else {
        0.0
    }
}

fn can_merge_multiline_segment(
    upper_left: f64,
    upper_right: f64,
    upper_height: f64,
    lower: &RawWord,
) -> bool {
    let lower_left = lower.x;
    let lower_right = lower.x + lower.width;

    if ranges_overlap(upper_left, upper_right, lower_left, lower_right) {
        return true;
    }

    let max_height = upper_height.max(lower.height);
    let upper_center = (upper_left + upper_right) * 0.5;
    let lower_center = (lower_left + lower_right) * 0.5;
    let center_delta = (upper_center - lower_center).abs();
    let center_tolerance = max_height * CENTER_ALIGNED_MERGE_FACTOR;
    if center_delta > center_tolerance {
        return false;
    }

    let gap = horizontal_gap(upper_left, upper_right, lower_left, lower_right);
    gap <= max_height * CENTER_ALIGNED_HORIZONTAL_GAP_FACTOR
}

fn group_words_into_blocks(words: &[OcrWord]) -> Vec<OcrWord> {
    if words.is_empty() {
        return vec![];
    }

    let mut raw_words = words
        .iter()
        .map(|word| RawWord {
            text: word.text.clone(),
            x: word.x,
            y: word.y,
            width: word.width,
            height: word.height,
        })
        .collect::<Vec<_>>();

    raw_words.sort_by(|left, right| {
        left.y
            .partial_cmp(&right.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left.x
                    .partial_cmp(&right.x)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    let mut lines: Vec<Vec<RawWord>> = Vec::new();
    let mut line_centers: Vec<f64> = Vec::new();
    let mut line_heights: Vec<f64> = Vec::new();

    for word in raw_words {
        let center_y = word.y + word.height * 0.5;
        let mut found_line = None;
        for (index, line_center) in line_centers.iter().enumerate() {
            let line_height = line_heights[index].max(word.height);
            if (center_y - *line_center).abs() <= line_height * SAME_LINE_VERTICAL_FACTOR {
                found_line = Some(index);
                break;
            }
        }

        if let Some(index) = found_line {
            let line = &mut lines[index];
            let prev_len = line.len() as f64;
            line.push(word.clone());
            line_centers[index] = (line_centers[index] * prev_len + center_y) / (prev_len + 1.0);
            line_heights[index] = line_heights[index].max(word.height);
        } else {
            lines.push(vec![word.clone()]);
            line_centers.push(center_y);
            line_heights.push(word.height);
        }
    }

    for line in &mut lines {
        line.sort_by(|left, right| {
            left.x
                .partial_cmp(&right.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let mut line_segments = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }

        let avg_height = line.iter().map(|word| word.height).sum::<f64>() / line.len() as f64;
        let max_gap = avg_height * HORIZONTAL_WORD_GAP_FACTOR;

        let mut current: Option<RawWord> = None;
        for word in line {
            match current.as_mut() {
                Some(segment) => {
                    let segment_right = segment.x + segment.width;
                    let gap = word.x - segment_right;
                    if gap <= max_gap {
                        segment.text.push(' ');
                        segment.text.push_str(&word.text);
                        let right = (segment.x + segment.width).max(word.x + word.width);
                        let bottom = (segment.y + segment.height).max(word.y + word.height);
                        segment.x = segment.x.min(word.x);
                        segment.y = segment.y.min(word.y);
                        segment.width = right - segment.x;
                        segment.height = bottom - segment.y;
                    } else {
                        line_segments.push(segment.clone());
                        current = Some(word.clone());
                    }
                }
                None => {
                    current = Some(word.clone());
                }
            }
        }
        if let Some(segment) = current {
            line_segments.push(segment);
        }
    }

    line_segments.sort_by(|left, right| {
        left.y
            .partial_cmp(&right.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left.x
                    .partial_cmp(&right.x)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    let mut consumed = vec![false; line_segments.len()];
    let mut merged_blocks = Vec::new();

    for index in 0..line_segments.len() {
        if consumed[index] {
            continue;
        }

        let first = &line_segments[index];
        consumed[index] = true;

        let mut merged_text = first.text.clone();
        let mut merged_x = first.x;
        let mut merged_y = first.y;
        let mut merged_right = first.x + first.width;
        let mut merged_bottom = first.y + first.height;

        let mut merged_line_count = 1usize;
        while merged_line_count < MAX_MERGED_LINES {
            let merged_height = merged_bottom - merged_y;
            let mut next_line_index = None;

            for next_index in (index + 1)..line_segments.len() {
                if consumed[next_index] {
                    continue;
                }

                let next = &line_segments[next_index];
                let next_bottom = next.y + next.height;
                let vertical_gap = horizontal_gap(merged_y, merged_bottom, next.y, next_bottom);
                if vertical_gap > merged_height.max(next.height) * MERGE_LINE_VERTICAL_FACTOR {
                    if next.y >= merged_bottom {
                        break;
                    }
                    continue;
                }

                if can_merge_multiline_segment(merged_x, merged_right, merged_height, next) {
                    next_line_index = Some(next_index);
                    break;
                }
            }

            let Some(next_index) = next_line_index else {
                break;
            };

            let next = &line_segments[next_index];
            consumed[next_index] = true;
            merged_text.push(' ');
            merged_text.push_str(&next.text);
            merged_x = merged_x.min(next.x);
            merged_y = merged_y.min(next.y);
            merged_right = merged_right.max(next.x + next.width);
            merged_bottom = merged_bottom.max(next.y + next.height);
            merged_line_count += 1;
        }

        merged_blocks.push(OcrWord {
            text: merged_text,
            x: merged_x,
            y: merged_y,
            width: merged_right - merged_x,
            height: merged_bottom - merged_y,
            slug: None,
            mapping_confidence: None,
            market_median: None,
            market_median_from_current_offers: None,
        });
    }

    merged_blocks
}
