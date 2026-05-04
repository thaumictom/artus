//! Tessdata resolution and word-grouping geometry.
//!
//! After Tesseract produces individual word bounding boxes, this module groups
//! them into logical lines and multi-line blocks using spatial heuristics.

use std::env;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};

use crate::error::{AppError, AppResult};
use crate::ocr::OcrWord;
use crate::store_ext::SettingsExt;

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

// ── Word grouping using connected components algorithm ────────────────────────

pub fn group_words<R: Runtime>(app: &AppHandle<R>, words: Vec<OcrWord>) -> Vec<OcrWord> {
    if words.is_empty() {
        return Vec::new();
    }

    // =====================================================================
    // TUNING PARAMETERS (Multipliers based on median text height)
    // =====================================================================

    let max_x_gap_multiplier = app.get_setting_f64("ocr_max_x_gap_multiplier", 1.0);
    let max_y_gap_multiplier = app.get_setting_f64("ocr_max_y_gap_multiplier", 2.0);
    let vertical_column_tolerance = app.get_setting_f64("ocr_vertical_column_tolerance", 2.5);
    let row_bucket_y_tolerance = app.get_setting_f64("ocr_row_bucket_y_tolerance", 0.6);

    // =====================================================================

    // 1. Calculate median height to make thresholds scale-independent
    let mut heights: Vec<f64> = words.iter().map(|w| w.height).collect();
    heights.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_h = heights[heights.len() / 2];

    let max_x_gap = median_h * max_x_gap_multiplier;
    let max_y_gap = median_h * max_y_gap_multiplier;
    let col_tolerance = median_h * vertical_column_tolerance;
    let row_tolerance = median_h * row_bucket_y_tolerance;

    let n = words.len();
    let mut adj = vec![vec![]; n];

    // 2. Build adjacency list based on bounding box proximity
    for i in 0..n {
        for j in (i + 1)..n {
            let a = &words[i];
            let b = &words[j];

            // Are they side-by-side on the same visual line?
            let horizontal_match = (a.y - b.y).abs() <= row_tolerance
                && (a.x + a.width - b.x).abs().min((b.x + b.width - a.x).abs()) < max_x_gap;

            // Are they stacked vertically within the same center-aligned column?
            let vertical_match = (a.x + a.width / 2.0 - (b.x + b.width / 2.0)).abs()
                < col_tolerance
                && (a.y + a.height - b.y)
                    .abs()
                    .min((b.y + b.height - a.y).abs())
                    < max_y_gap;

            if horizontal_match || vertical_match {
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }

    // 3. Find connected components (DFS)
    let mut visited = vec![false; n];
    let mut grouped_items = Vec::new();

    for i in 0..n {
        if !visited[i] {
            let mut component = Vec::new();
            let mut stack = vec![i];

            while let Some(node) = stack.pop() {
                if !visited[node] {
                    visited[node] = true;
                    component.push(node);
                    for &neighbor in &adj[node] {
                        if !visited[neighbor] {
                            stack.push(neighbor);
                        }
                    }
                }
            }

            // 4. Sort component purely by Y first
            component
                .sort_by(|&a_idx, &b_idx| words[a_idx].y.partial_cmp(&words[b_idx].y).unwrap());

            // 5. Bucket into distinct rows based on the Y-tolerance
            let mut rows: Vec<Vec<usize>> = Vec::new();
            let mut current_row: Vec<usize> = Vec::new();
            let mut current_row_y = -1.0;

            for &node in &component {
                let w = &words[node];

                if current_row.is_empty() || (w.y - current_row_y).abs() <= row_tolerance {
                    current_row.push(node);
                    if current_row_y < 0.0 {
                        current_row_y = w.y;
                    }
                } else {
                    rows.push(current_row);
                    current_row = vec![node];
                    current_row_y = w.y;
                }
            }
            if !current_row.is_empty() {
                rows.push(current_row);
            }

            // 6. Sort each row left-to-right, then assemble final reading order
            let mut ordered_component = Vec::new();
            for mut row in rows {
                row.sort_by(|&a_idx, &b_idx| words[a_idx].x.partial_cmp(&words[b_idx].x).unwrap());
                ordered_component.extend(row);
            }

            // 7. Merge bounding boxes and text
            let mut merged_text = String::new();
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;

            for (idx, &node) in ordered_component.iter().enumerate() {
                let w = &words[node];
                if idx > 0 {
                    merged_text.push(' ');
                }
                merged_text.push_str(&w.text);

                min_x = min_x.min(w.x);
                min_y = min_y.min(w.y);
                max_x = max_x.max(w.x + w.width);
                max_y = max_y.max(w.y + w.height);
            }

            grouped_items.push(OcrWord::new(
                merged_text,
                min_x,
                min_y,
                max_x - min_x,
                max_y - min_y,
            ));
        }
    }

    grouped_items
}
