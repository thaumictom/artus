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

// ── Segmentation ──────────────────────────────────────────────────────────────

/// Segments a binary GrayImage (0 = text, 255 = background) into bounding boxes
/// using X/Y intensity projections (histograms).
pub fn segment_image(image: &image::GrayImage) -> Vec<(u32, u32, u32, u32)> {
    let width = image.width();
    let height = image.height();

    // 1. Calculate row histograms (sum of text pixels per row)
    let mut row_hist = vec![0u32; height as usize];
    for y in 0..height {
        let mut text_pixels = 0;
        for x in 0..width {
            if image.get_pixel(x, y)[0] == 0 {
                text_pixels += 1;
            }
        }
        row_hist[y as usize] = text_pixels;
    }

    // 2. Find row ranges (valleys)
    let mut y_ranges = Vec::new();
    let mut in_text = false;
    let mut start_y = 0;
    let mut zero_count = 0;
    let max_y_gap = 10;

    for y in 0..height {
        if row_hist[y as usize] > 0 {
            if !in_text {
                in_text = true;
                start_y = y;
            }
            zero_count = 0;
        } else {
            if in_text {
                zero_count += 1;
                if zero_count > max_y_gap {
                    in_text = false;
                    y_ranges.push((start_y, y - zero_count));
                }
            }
        }
    }
    if in_text {
        y_ranges.push((start_y, height - 1));
    }

    // 3. For each y_range, find x_ranges (columns with text)
    let mut bounding_boxes = Vec::new();
    let max_x_gap = 30;

    for (start_y, end_y) in y_ranges {
        let mut col_hist = vec![0u32; width as usize];
        for y in start_y..=end_y {
            for x in 0..width {
                if image.get_pixel(x, y)[0] == 0 {
                    col_hist[x as usize] += 1;
                }
            }
        }

        let mut in_text_x = false;
        let mut start_x = 0;
        let mut zero_count_x = 0;

        for x in 0..width {
            if col_hist[x as usize] > 0 {
                if !in_text_x {
                    in_text_x = true;
                    start_x = x;
                }
                zero_count_x = 0;
            } else {
                if in_text_x {
                    zero_count_x += 1;
                    if zero_count_x > max_x_gap {
                        in_text_x = false;
                        bounding_boxes.push((start_x, start_y, (x - zero_count_x) - start_x, end_y - start_y + 1));
                    }
                }
            }
        }
        if in_text_x {
            bounding_boxes.push((start_x, start_y, width - start_x, end_y - start_y + 1));
        }
    }

    // Filter out extremely small boxes (noise) and add 4px padding
    let pad = 4;
    bounding_boxes
        .into_iter()
        .filter(|&(_, _, w, h)| w > 1 && h > 1)
        .map(|(x, y, w, h)| {
            let px = x.saturating_sub(pad);
            let py = y.saturating_sub(pad);
            let pw = (x + w + pad * 2).min(width.saturating_sub(px));
            let ph = (y + h + pad * 2).min(height.saturating_sub(py));
            (px, py, pw, ph)
        })
        .collect()
}
