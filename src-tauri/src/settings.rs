use std::path::Path;

use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

const WARFRAME_LOG_FILENAME: &str = "EE.log";
const SETTINGS_STORE_PATH: &str = "settings.json";

#[tauri::command]
pub fn get_warframe_log_path<R: Runtime>(app: AppHandle<R>) -> Result<String, String> {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;
    Ok(store
        .get("warframe_log_path")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default())
}

#[tauri::command]
pub fn set_warframe_log_path<R: Runtime>(app: AppHandle<R>, path: String) -> Result<String, String> {
    let normalized_path = validate_warframe_log_path(&path)?;

    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;
    store.set("warframe_log_path", normalized_path.clone());
    let _ = store.save();

    Ok(normalized_path)
}

fn validate_warframe_log_path(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let path = Path::new(trimmed);
    if !path.exists() || !path.is_file() {
        return Err("warframe log path must point to an existing file".to_string());
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(format!(
            "warframe log path must point to a file named {WARFRAME_LOG_FILENAME}"
        ));
    };

    if file_name != WARFRAME_LOG_FILENAME {
        return Err(format!(
            "warframe log file must be named {WARFRAME_LOG_FILENAME}"
        ));
    }

    Ok(trimmed.to_string())
}
