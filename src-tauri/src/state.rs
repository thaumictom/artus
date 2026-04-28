use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub const HOTKEY_ACTION_SCREENSHOT: &str = "screenshot";
pub const DEFAULT_SCREENSHOT_HOTKEY: &str = "Ctrl+Home";
pub const HOTKEY_ACTION_SCREENSHOT_ADD_TO_INVENTORY: &str = "screenshot_add_inventory";
pub const DEFAULT_SCREENSHOT_ADD_TO_INVENTORY_HOTKEY: &str = "Ctrl+Shift+Home";

fn default_hotkeys() -> HashMap<String, String> {
    HashMap::from([
        (
            HOTKEY_ACTION_SCREENSHOT.to_string(),
            DEFAULT_SCREENSHOT_HOTKEY.to_string(),
        ),
        (
            HOTKEY_ACTION_SCREENSHOT_ADD_TO_INVENTORY.to_string(),
            DEFAULT_SCREENSHOT_ADD_TO_INVENTORY_HOTKEY.to_string(),
        ),
    ])
}

#[derive(Debug, Clone)]
pub struct OcrDictionaryEntry {
    pub name: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub normalized_name: String,
    pub ducats: Option<u64>,
    pub vaulted: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TradeablePriceEntry {
    pub median: f64,
    pub used_current_offer_fallback: bool,
    pub trades_24h: Option<f64>,
    pub moving_avg: Option<f64>,
    pub ducats: Option<u64>,
}

pub struct AppState {
    pub hotkeys: Mutex<HashMap<String, String>>,
    pub overlay_sequence: Mutex<u64>,
    pub overlay_duration_secs: Mutex<u64>,
    pub overlay_toggle_mode: Mutex<bool>,
    pub overlay_toggle_in_flight: AtomicBool,
    pub warframe_log_path: Mutex<String>,
    pub ocr_theme: Mutex<String>,
    pub ocr_target_rgb: Mutex<[u8; 3]>,
    pub ocr_dictionary: Mutex<Vec<OcrDictionaryEntry>>,
    pub ocr_tradeable_prices: Mutex<HashMap<String, TradeablePriceEntry>>,
    pub ocr_dictionary_mapping_enabled: Mutex<bool>,
    pub ocr_dictionary_match_threshold: Mutex<f64>,
    pub relic_reward_detection: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkeys: Mutex::new(default_hotkeys()),
            overlay_sequence: Mutex::new(0),
            overlay_duration_secs: Mutex::new(10),
            overlay_toggle_mode: Mutex::new(false),
            overlay_toggle_in_flight: AtomicBool::new(false),
            warframe_log_path: Mutex::new(String::new()),
            ocr_theme: Mutex::new("EQUINOX".to_string()),
            ocr_target_rgb: Mutex::new([158, 159, 167]),
            ocr_dictionary: Mutex::new(Vec::new()),
            ocr_tradeable_prices: Mutex::new(HashMap::new()),
            ocr_dictionary_mapping_enabled: Mutex::new(true),
            ocr_dictionary_match_threshold: Mutex::new(0.62),
            relic_reward_detection: Mutex::new(false),
        }
    }
}
