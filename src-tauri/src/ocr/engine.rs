//! Tessdata resolution and word-grouping geometry.
//!
//! After Tesseract produces individual word bounding boxes, this module groups
//! them into logical lines and multi-line blocks using spatial heuristics.

use std::env;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};


use crate::error::{AppError, AppResult};

#[cfg(target_os = "windows")]
use super::{EMBEDDED_TRAINEDDATA_BYTES, EMBEDDED_TRAINEDDATA_FILENAME};

// ── Tessdata resolution ───────────────────────────────────────────────────────

/// Locates the `tessdata` directory, checking (in order):
/// 1. Embedded traineddata extracted to app-data (Windows only)
/// 2. Tauri resource directory
/// 3. Executable directory
/// 4. Current working directory
pub fn resolve_tessdata<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    let mut checked_paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        match resolve_embedded_tessdata(app) {
            Ok(path) if has_traineddata_files(&path) => return Ok(path),
            Ok(path) => checked_paths.push(path.display().to_string()),
            Err(err) => checked_paths.push(format!("embedded: {err}")),
        }
    }

    let mut candidates = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("tessdata"));
        candidates.push(resource_dir);
    }
    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            candidates.push(exe_dir.join("tessdata"));
            candidates.push(exe_dir.to_path_buf());
        }
    }
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("src-tauri").join("tessdata"));
        candidates.push(cwd.join("tessdata"));
    }

    for candidate in candidates {
        if has_traineddata_files(&candidate) {
            return Ok(candidate);
        }
        checked_paths.push(candidate.display().to_string());
    }

    Err(AppError::msg(format!(
        "could not find tessdata directory (checked: {})",
        checked_paths.join(", ")
    )))
}

/// On Windows, extract the compiled-in traineddata to the app-data directory
/// so Tesseract can find it at a stable filesystem path.
#[cfg(target_os = "windows")]
fn resolve_embedded_tessdata<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| AppError::msg(format!("failed to resolve app data dir: {err}")))?;
    let tessdata_dir = app_data_dir.join("tessdata");

    std::fs::create_dir_all(&tessdata_dir)?;

    let traineddata_path = tessdata_dir.join(EMBEDDED_TRAINEDDATA_FILENAME);
    let should_write = std::fs::metadata(&traineddata_path)
        .map(|meta| meta.len() != EMBEDDED_TRAINEDDATA_BYTES.len() as u64)
        .unwrap_or(true);

    if should_write {
        std::fs::write(&traineddata_path, EMBEDDED_TRAINEDDATA_BYTES)?;
    }

    Ok(tessdata_dir)
}

/// Returns `true` if the directory contains at least one `.traineddata` file.
fn has_traineddata_files(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("traineddata"))
            })
        })
        .unwrap_or(false)
}

