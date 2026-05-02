//! Tessdata resolution and word-grouping geometry.
//!
//! After Tesseract produces individual word bounding boxes, this module groups
//! them into logical lines and multi-line blocks using spatial heuristics.

use std::env;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};

use super::{
    OcrWord, CENTER_ALIGNED_HORIZONTAL_GAP_FACTOR, CENTER_ALIGNED_MERGE_FACTOR,
    HORIZONTAL_WORD_GAP_FACTOR, MAX_MERGED_LINES, MERGE_LINE_VERTICAL_FACTOR,
    SAME_LINE_VERTICAL_FACTOR,
};
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

// ── Geometric helpers ─────────────────────────────────────────────────────────

/// Intermediate representation for word geometry during grouping.
#[derive(Debug, Clone)]
struct RawWord {
    text: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Returns `true` if the two ranges `[a_start, a_end]` and `[b_start, b_end]` overlap.
fn ranges_overlap(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> bool {
    a_start.max(b_start) <= a_end.min(b_end)
}

/// Returns the horizontal gap between two non-overlapping ranges, or 0 if they overlap.
fn horizontal_gap(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> f64 {
    if a_end < b_start {
        b_start - a_end
    } else if b_end < a_start {
        a_start - b_end
    } else {
        0.0
    }
}

/// Compare float values for sorting, treating NaN as equal.
fn f64_cmp(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

/// Decides whether a lower line segment can be merged into an upper multi-line block.
fn can_merge_multiline_segment(
    upper_left: f64,
    upper_right: f64,
    upper_height: f64,
    lower: &RawWord,
) -> bool {
    let lower_left = lower.x;
    let lower_right = lower.x + lower.width;

    // Direct horizontal overlap → always merge
    if ranges_overlap(upper_left, upper_right, lower_left, lower_right) {
        return true;
    }

    // Center-aligned tolerance check
    let max_height = upper_height.max(lower.height);
    let upper_center = (upper_left + upper_right) * 0.5;
    let lower_center = (lower_left + lower_right) * 0.5;
    let center_delta = (upper_center - lower_center).abs();

    if center_delta > max_height * CENTER_ALIGNED_MERGE_FACTOR {
        return false;
    }

    let gap = horizontal_gap(upper_left, upper_right, lower_left, lower_right);
    gap <= max_height * CENTER_ALIGNED_HORIZONTAL_GAP_FACTOR
}

// ── Word grouping pipeline ────────────────────────────────────────────────────

/// Groups individual OCR words into logical blocks by:
/// 1. Assigning words to horizontal lines based on vertical proximity.
/// 2. Merging adjacent words on the same line into segments.
/// 3. Merging vertically adjacent segments into multi-line blocks.
pub fn group_words_into_blocks(words: &[OcrWord]) -> Vec<OcrWord> {
    if words.is_empty() {
        return Vec::new();
    }

    // Convert to mutable intermediate representation, sorted top-to-bottom
    let mut raw_words: Vec<RawWord> = words
        .iter()
        .map(|w| RawWord {
            text: w.text.clone(),
            x: w.x,
            y: w.y,
            width: w.width,
            height: w.height,
        })
        .collect();

    raw_words.sort_by(|a, b| f64_cmp(a.y, b.y).then(f64_cmp(a.x, b.x)));

    // ── Step 1: Cluster words into horizontal lines ──
    let mut lines: Vec<Vec<RawWord>> = Vec::new();
    let mut line_centers: Vec<f64> = Vec::new();
    let mut line_heights: Vec<f64> = Vec::new();

    for word in raw_words {
        let center_y = word.y + word.height * 0.5;

        let existing_line = line_centers.iter().enumerate().find(|(i, &lc)| {
            let lh = line_heights[*i].max(word.height);
            (center_y - lc).abs() <= lh * SAME_LINE_VERTICAL_FACTOR
        });

        if let Some((idx, _)) = existing_line {
            let prev_count = lines[idx].len() as f64;
            lines[idx].push(word.clone());
            // Running average of vertical centers
            line_centers[idx] = (line_centers[idx] * prev_count + center_y) / (prev_count + 1.0);
            line_heights[idx] = line_heights[idx].max(word.height);
        } else {
            lines.push(vec![word.clone()]);
            line_centers.push(center_y);
            line_heights.push(word.height);
        }
    }

    // Sort words within each line left-to-right
    for line in &mut lines {
        line.sort_by(|a, b| f64_cmp(a.x, b.x));
    }

    // ── Step 2: Merge adjacent words into line segments ──
    let mut line_segments = Vec::new();
    for line in &lines {
        if line.is_empty() {
            continue;
        }

        let avg_height = line.iter().map(|w| w.height).sum::<f64>() / line.len() as f64;
        let max_gap = avg_height * HORIZONTAL_WORD_GAP_FACTOR;

        let mut current: Option<RawWord> = None;
        for word in line {
            match current.as_mut() {
                Some(seg) => {
                    let seg_right = seg.x + seg.width;
                    if word.x - seg_right <= max_gap {
                        // Merge: extend the segment
                        seg.text.push(' ');
                        seg.text.push_str(&word.text);
                        let right = (seg.x + seg.width).max(word.x + word.width);
                        let bottom = (seg.y + seg.height).max(word.y + word.height);
                        seg.x = seg.x.min(word.x);
                        seg.y = seg.y.min(word.y);
                        seg.width = right - seg.x;
                        seg.height = bottom - seg.y;
                    } else {
                        line_segments.push(seg.clone());
                        current = Some(word.clone());
                    }
                }
                None => current = Some(word.clone()),
            }
        }
        if let Some(seg) = current {
            line_segments.push(seg);
        }
    }

    line_segments.sort_by(|a, b| f64_cmp(a.y, b.y).then(f64_cmp(a.x, b.x)));

    // ── Step 3: Merge vertically adjacent segments into multi-line blocks ──
    let mut consumed = vec![false; line_segments.len()];
    let mut merged_blocks = Vec::new();

    for i in 0..line_segments.len() {
        if consumed[i] {
            continue;
        }
        consumed[i] = true;

        let first = &line_segments[i];
        let mut text = first.text.clone();
        let mut left = first.x;
        let mut top = first.y;
        let mut right = first.x + first.width;
        let mut bottom = first.y + first.height;
        let mut line_count = 1usize;

        while line_count < MAX_MERGED_LINES {
            let height = bottom - top;

            let next_idx = ((i + 1)..line_segments.len()).find(|&j| {
                if consumed[j] {
                    return false;
                }
                let next = &line_segments[j];
                let next_bottom = next.y + next.height;
                let vgap = horizontal_gap(top, bottom, next.y, next_bottom);

                if vgap > height.max(next.height) * MERGE_LINE_VERTICAL_FACTOR {
                    return false;
                }
                can_merge_multiline_segment(left, right, height, next)
            });

            let Some(j) = next_idx else { break };

            let next = &line_segments[j];
            consumed[j] = true;
            text.push(' ');
            text.push_str(&next.text);
            left = left.min(next.x);
            top = top.min(next.y);
            right = right.max(next.x + next.width);
            bottom = bottom.max(next.y + next.height);
            line_count += 1;
        }

        merged_blocks.push(OcrWord::new(text, left, top, right - left, bottom - top));
    }

    merged_blocks
}
