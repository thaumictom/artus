//! Global application state shared across all modules via `tauri::Manager::state()`.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::ocr::dictionary::{OcrDictionaryEntry, TradeablePriceEntry};
use crate::ocr::mastery::MasteryDictionaryEntry;

/// Shared mutable state for the Tauri application.
///
/// Every field is wrapped in a synchronization primitive so that
/// concurrent Tauri command handlers and background tasks can access
/// it safely.
pub struct AppState {
    /// Last normal Artus window size in logical pixels.
    pub artus_window_size: Mutex<(f64, f64)>,
    /// Mapping of action name → shortcut string (e.g. "screenshot" → "Ctrl+Home").
    pub hotkeys: Mutex<HashMap<String, String>>,

    /// Monotonic counter used to cancel stale overlay auto-hide timers.
    pub overlay_sequence: Mutex<u64>,

    /// Prevents overlapping toggle-overlay hotkey invocations.
    pub overlay_toggle_in_flight: AtomicBool,
    /// Overlay navigation shortcuts are active only after OCR results are shown.
    pub overlay_controls_active: AtomicBool,
    /// Routes Escape, the configured confirm key, and Space to the listing dialog while open.
    pub overlay_listing_dialog_open: AtomicBool,

    /// True if the current or pending overlay capture was triggered by relic rewards.
    pub overlay_is_relic_mode: AtomicBool,

    /// True if the overlay was visibly shown before focus was lost.
    pub overlay_was_visible: AtomicBool,

    /// Parsed theme name → RGB color from `theme_colors.toml`.
    pub ocr_theme_colors: Mutex<HashMap<String, [u8; 3]>>,

    /// OCR dictionary entries fetched from the remote API on startup.
    pub ocr_dictionary: Mutex<Vec<OcrDictionaryEntry>>,
    /// Catalog payloads shared by all windows and lookup commands.
    pub catalogs: Mutex<crate::data::api::CatalogCache>,
    /// Prevent overlapping scheduled and manual catalog refreshes.
    pub catalog_refresh: Mutex<()>,

    /// Full masterable item and component names, including non-tradeable gear.
    pub mastery_dictionary: Mutex<Vec<MasteryDictionaryEntry>>,

    /// Median prices keyed by item slug, fetched from the remote API.
    pub ocr_tradeable_prices: Mutex<HashMap<String, TradeablePriceEntry>>,

    /// `true` while the Warframe window is the active foreground window.
    pub warframe_focused: AtomicBool,

    /// `true` while a Warframe process is detected in the system process list.
    pub warframe_running: AtomicBool,

    /// Shared HTTP client for all outgoing requests.
    pub http_client: reqwest::Client,
    /// Rotation-aware wiki requests are serialized across windows; snapshots are also saved locally.
    pub wiki_offerings:
        tokio::sync::Mutex<HashMap<String, crate::data::wiki_offerings::WikiOfferings>>,
    /// Oracle rotations and display metadata, shared by all windows for this session.
    pub oracle_bounties: tokio::sync::Mutex<crate::data::oracle::OracleCache>,
    /// Short-lived warframe.market item orders and historical statistics.
    pub market_responses: Mutex<crate::market::MarketResponseCaches>,
    /// warframe.market credentials are never kept here; only the active JWT and socket handle.
    pub market_session: Mutex<Option<crate::market::account::MarketSession>>,
    /// Serializes manual, scheduled, and exit-time market status changes.
    pub market_status_operation: tokio::sync::Mutex<()>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            artus_window_size: Mutex::new((800.0, 600.0)),
            hotkeys: Mutex::new(HashMap::new()),
            overlay_sequence: Mutex::new(0),
            overlay_toggle_in_flight: AtomicBool::new(false),
            overlay_controls_active: AtomicBool::new(false),
            overlay_listing_dialog_open: AtomicBool::new(false),
            overlay_is_relic_mode: AtomicBool::new(false),
            overlay_was_visible: AtomicBool::new(false),
            ocr_theme_colors: Mutex::new(HashMap::new()),
            ocr_dictionary: Mutex::new(Vec::new()),
            catalogs: Mutex::new(crate::data::api::CatalogCache::default()),
            catalog_refresh: Mutex::new(()),
            mastery_dictionary: Mutex::new(Vec::new()),
            ocr_tradeable_prices: Mutex::new(HashMap::new()),
            warframe_focused: AtomicBool::new(false),
            warframe_running: AtomicBool::new(false),
            http_client: reqwest::Client::builder()
                .user_agent(crate::data::api::USER_AGENT)
                .build()
                .expect("shared HTTP client"),
            market_responses: Mutex::new(crate::market::MarketResponseCaches::default()),
            wiki_offerings: tokio::sync::Mutex::new(HashMap::new()),
            oracle_bounties: tokio::sync::Mutex::new(crate::data::oracle::OracleCache::default()),
            market_session: Mutex::new(None),
            market_status_operation: tokio::sync::Mutex::new(()),
        }
    }
}
