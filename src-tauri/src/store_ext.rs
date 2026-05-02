//! Convenience accessors for the persisted settings store.
//!
//! Instead of repeating `app.store("settings.json").ok().and_then(…)` everywhere,
//! modules can call `app.get_setting_bool("key", false)` etc.

use std::collections::HashMap;

use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

/// Path used by the Tauri store plugin for user settings.
pub const SETTINGS_STORE_PATH: &str = "settings.json";

/// Extension methods for reading typed values from the settings store.
pub trait SettingsExt {
    fn get_setting_bool(&self, key: &str, default: bool) -> bool;
    fn get_setting_u64(&self, key: &str, default: u64) -> u64;
    fn get_setting_f64(&self, key: &str, default: f64) -> f64;
    fn get_setting_string(&self, key: &str, default: &str) -> String;
    fn get_setting_json_map(&self, key: &str) -> Option<HashMap<String, String>>;
}

impl<R: Runtime> SettingsExt for AppHandle<R> {
    fn get_setting_bool(&self, key: &str, default: bool) -> bool {
        self.store(SETTINGS_STORE_PATH)
            .ok()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    fn get_setting_u64(&self, key: &str, default: u64) -> u64 {
        self.store(SETTINGS_STORE_PATH)
            .ok()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_u64())
            .unwrap_or(default)
    }

    fn get_setting_f64(&self, key: &str, default: f64) -> f64 {
        self.store(SETTINGS_STORE_PATH)
            .ok()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_f64())
            .unwrap_or(default)
    }

    fn get_setting_string(&self, key: &str, default: &str) -> String {
        self.store(SETTINGS_STORE_PATH)
            .ok()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| default.to_string())
    }

    fn get_setting_json_map(&self, key: &str) -> Option<HashMap<String, String>> {
        self.store(SETTINGS_STORE_PATH)
            .ok()
            .and_then(|s| s.get(key))
            .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v).ok())
    }
}
