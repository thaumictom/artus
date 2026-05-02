//! Settings validation commands.

use std::path::Path;

use crate::error::{AppError, AppResult};

/// Expected filename for the Warframe log.
const WARFRAME_LOG_FILENAME: &str = "EE.log";

/// Validates that a path points to an existing file named `EE.log`.
/// Returns the cleaned path on success, or an empty string if the input is blank.
#[tauri::command]
pub fn validate_warframe_log_path(path: String) -> AppResult<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let p = Path::new(trimmed);
    if !p.exists() || !p.is_file() {
        return Err(AppError::msg(
            "warframe log path must point to an existing file",
        ));
    }

    let file_name = p.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
        AppError::msg(format!(
            "warframe log path must point to a file named {WARFRAME_LOG_FILENAME}"
        ))
    })?;

    if file_name != WARFRAME_LOG_FILENAME {
        return Err(AppError::msg(format!(
            "warframe log file must be named {WARFRAME_LOG_FILENAME}"
        )));
    }

    Ok(trimmed.to_string())
}
