use std::sync::Mutex;

pub struct AppState {
    pub hotkey: Mutex<String>,
    pub overlay_sequence: Mutex<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkey: Mutex::new("Home".to_string()),
            overlay_sequence: Mutex::new(0),
        }
    }
}
