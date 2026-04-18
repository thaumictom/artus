use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub struct AppState {
    pub hotkey: Mutex<String>,
    pub overlay_sequence: Mutex<u64>,
    pub overlay_duration_secs: Mutex<u64>,
    pub overlay_toggle_mode: Mutex<bool>,
    pub overlay_toggle_in_flight: AtomicBool,
    pub ocr_theme: Mutex<String>,
    pub ocr_target_rgb: Mutex<[u8; 3]>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkey: Mutex::new("Home".to_string()),
            overlay_sequence: Mutex::new(0),
            overlay_duration_secs: Mutex::new(10),
            overlay_toggle_mode: Mutex::new(false),
            overlay_toggle_in_flight: AtomicBool::new(false),
            ocr_theme: Mutex::new("EQUINOX".to_string()),
            ocr_target_rgb: Mutex::new([158, 159, 167]),
        }
    }
}
