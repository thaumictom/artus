//! Global application state shared across all modules via `tauri::Manager::state()`.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::ocr::dictionary::{OcrDictionaryEntry, TradeablePriceEntry};

/// Shared mutable state for the Tauri application.
///
/// Every field is wrapped in a synchronization primitive so that
/// concurrent Tauri command handlers and background tasks can access
/// it safely.
pub struct AppState {
    /// Mapping of action name → shortcut string (e.g. "screenshot" → "Ctrl+Home").
    pub hotkeys: Mutex<HashMap<String, String>>,

    /// Monotonic counter used to cancel stale overlay auto-hide timers.
    pub overlay_sequence: Mutex<u64>,

    /// Prevents overlapping toggle-overlay hotkey invocations.
    pub overlay_toggle_in_flight: AtomicBool,

    /// Parsed theme name → RGB color from `theme_colors.toml`.
    pub ocr_theme_colors: Mutex<HashMap<String, [u8; 3]>>,

    /// OCR dictionary entries fetched from the remote API on startup.
    pub ocr_dictionary: Mutex<Vec<OcrDictionaryEntry>>,

    /// Median prices keyed by item slug, fetched from the remote API.
    pub ocr_tradeable_prices: Mutex<HashMap<String, TradeablePriceEntry>>,

    /// `true` while the Warframe window is the active foreground window.
    pub warframe_focused: AtomicBool,

    /// `true` while a Warframe process is detected in the system process list.
    pub warframe_running: AtomicBool,

    /// Shared HTTP client for all outgoing requests.
    pub http_client: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkeys: Mutex::new(HashMap::new()),
            overlay_sequence: Mutex::new(0),
            overlay_toggle_in_flight: AtomicBool::new(false),
            ocr_theme_colors: Mutex::new(HashMap::new()),
            ocr_dictionary: Mutex::new(Vec::new()),
            ocr_tradeable_prices: Mutex::new(HashMap::new()),
            warframe_focused: AtomicBool::new(false),
            warframe_running: AtomicBool::new(false),
            http_client: reqwest::Client::new(),
        }
    }
}
