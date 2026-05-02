use std::path::Path;

const WARFRAME_LOG_FILENAME: &str = "EE.log";

#[tauri::command]
pub fn validate_warframe_log_path(path: String) -> Result<String, String> {
    let trimmed = path.trim();
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
