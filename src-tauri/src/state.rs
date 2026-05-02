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
    pub is_custom: bool,
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
    pub overlay_toggle_in_flight: AtomicBool,
    pub warframe_log_path: Mutex<String>,
    pub ocr_theme_colors: Mutex<HashMap<String, [u8; 3]>>,
    pub ocr_dictionary: Mutex<Vec<OcrDictionaryEntry>>,
    pub ocr_tradeable_prices: Mutex<HashMap<String, TradeablePriceEntry>>,
    pub warframe_focused: AtomicBool,
    pub warframe_running: AtomicBool,
    pub http_client: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkeys: Mutex::new(default_hotkeys()),
            overlay_sequence: Mutex::new(0),
            overlay_toggle_in_flight: AtomicBool::new(false),
            warframe_log_path: Mutex::new(String::new()),
            ocr_theme_colors: Mutex::new(HashMap::new()),
            ocr_dictionary: Mutex::new(Vec::new()),
            ocr_tradeable_prices: Mutex::new(HashMap::new()),
            warframe_focused: AtomicBool::new(false),
            warframe_running: AtomicBool::new(false),
            http_client: reqwest::Client::new(),
        }
    }
}
