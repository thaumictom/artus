use std::env;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};

use super::{
    OcrWord, SAME_LINE_VERTICAL_FACTOR, HORIZONTAL_WORD_GAP_FACTOR,
    MERGE_LINE_VERTICAL_FACTOR, MAX_MERGED_LINES, CENTER_ALIGNED_MERGE_FACTOR,
    CENTER_ALIGNED_HORIZONTAL_GAP_FACTOR,
};

#[cfg(target_os = "windows")]
use super::{EMBEDDED_TRAINEDDATA_BYTES, EMBEDDED_TRAINEDDATA_FILENAME};

#[derive(Debug, Clone)]
pub struct RawWord {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn resolve_tessdata<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let mut checked_paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        match resolve_embedded_tessdata(app) {
            Ok(path) if has_traineddata_files(&path) => return Ok(path),
            Ok(path) => checked_paths.push(path.display().to_string()),
            Err(err) => checked_paths.push(format!("embedded: {err}")),
        }
    }

    let mut candidates = vec![];

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

    Err(format!(
        "could not find tessdata directory (checked: {})",
        checked_paths.join(", ")
    ))
}

#[cfg(target_os = "windows")]
pub fn resolve_embedded_tessdata<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?;
    let tessdata_dir = app_data_dir.join("tessdata");

    std::fs::create_dir_all(&tessdata_dir)
        .map_err(|err| format!("failed to create embedded tessdata dir: {err}"))?;

    let traineddata_path = tessdata_dir.join(EMBEDDED_TRAINEDDATA_FILENAME);
    let should_write = std::fs::metadata(&traineddata_path)
        .map(|meta| meta.len() != EMBEDDED_TRAINEDDATA_BYTES.len() as u64)
        .unwrap_or(true);

    if should_write {
        std::fs::write(&traineddata_path, EMBEDDED_TRAINEDDATA_BYTES)
            .map_err(|err| format!("failed to write embedded traineddata: {err}"))?;
    }

    Ok(tessdata_dir)
}

pub fn has_traineddata_files(path: &Path) -> bool {
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

pub fn ranges_overlap(left_start: f64, left_end: f64, right_start: f64, right_end: f64) -> bool {
    left_start.max(right_start) <= left_end.min(right_end)
}

pub fn horizontal_gap(left_start: f64, left_end: f64, right_start: f64, right_end: f64) -> f64 {
    if left_end < right_start {
        right_start - left_end
    } else if right_end < left_start {
        left_start - right_end
    } else {
        0.0
    }
}

pub fn can_merge_multiline_segment(
    upper_left: f64,
    upper_right: f64,
    upper_height: f64,
    lower: &RawWord,
) -> bool {
    let lower_left = lower.x;
    let lower_right = lower.x + lower.width;

    if ranges_overlap(upper_left, upper_right, lower_left, lower_right) {
        return true;
    }

    let max_height = upper_height.max(lower.height);
    let upper_center = (upper_left + upper_right) * 0.5;
    let lower_center = (lower_left + lower_right) * 0.5;
    let center_delta = (upper_center - lower_center).abs();
    let center_tolerance = max_height * CENTER_ALIGNED_MERGE_FACTOR;
    if center_delta > center_tolerance {
        return false;
    }

    let gap = horizontal_gap(upper_left, upper_right, lower_left, lower_right);
    gap <= max_height * CENTER_ALIGNED_HORIZONTAL_GAP_FACTOR
}

pub fn group_words_into_blocks(words: &[OcrWord]) -> Vec<OcrWord> {
    if words.is_empty() {
        return vec![];
    }

    let mut raw_words = words
        .iter()
        .map(|word| RawWord {
            text: word.text.clone(),
            x: word.x,
            y: word.y,
            width: word.width,
            height: word.height,
        })
        .collect::<Vec<_>>();

    raw_words.sort_by(|left, right| {
        left.y
            .partial_cmp(&right.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left.x
                    .partial_cmp(&right.x)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    let mut lines: Vec<Vec<RawWord>> = Vec::new();
    let mut line_centers: Vec<f64> = Vec::new();
    let mut line_heights: Vec<f64> = Vec::new();

    for word in raw_words {
        let center_y = word.y + word.height * 0.5;
        let mut found_line = None;
        for (index, line_center) in line_centers.iter().enumerate() {
            let line_height = line_heights[index].max(word.height);
            if (center_y - *line_center).abs() <= line_height * SAME_LINE_VERTICAL_FACTOR {
                found_line = Some(index);
                break;
            }
        }

        if let Some(index) = found_line {
            let line = &mut lines[index];
            let prev_len = line.len() as f64;
            line.push(word.clone());
            line_centers[index] = (line_centers[index] * prev_len + center_y) / (prev_len + 1.0);
            line_heights[index] = line_heights[index].max(word.height);
        } else {
            lines.push(vec![word.clone()]);
            line_centers.push(center_y);
            line_heights.push(word.height);
        }
    }

    for line in &mut lines {
        line.sort_by(|left, right| {
            left.x
                .partial_cmp(&right.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let mut line_segments = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }

        let avg_height = line.iter().map(|word| word.height).sum::<f64>() / line.len() as f64;
        let max_gap = avg_height * HORIZONTAL_WORD_GAP_FACTOR;

        let mut current: Option<RawWord> = None;
        for word in line {
            match current.as_mut() {
                Some(segment) => {
                    let segment_right = segment.x + segment.width;
                    let gap = word.x - segment_right;
                    if gap <= max_gap {
                        segment.text.push(' ');
                        segment.text.push_str(&word.text);
                        let right = (segment.x + segment.width).max(word.x + word.width);
                        let bottom = (segment.y + segment.height).max(word.y + word.height);
                        segment.x = segment.x.min(word.x);
                        segment.y = segment.y.min(word.y);
                        segment.width = right - segment.x;
                        segment.height = bottom - segment.y;
                    } else {
                        line_segments.push(segment.clone());
                        current = Some(word.clone());
                    }
                }
                None => {
                    current = Some(word.clone());
                }
            }
        }
        if let Some(segment) = current {
            line_segments.push(segment);
        }
    }

    line_segments.sort_by(|left, right| {
        left.y
            .partial_cmp(&right.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left.x
                    .partial_cmp(&right.x)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    let mut consumed = vec![false; line_segments.len()];
    let mut merged_blocks = Vec::new();

    for index in 0..line_segments.len() {
        if consumed[index] {
            continue;
        }

        let first = &line_segments[index];
        consumed[index] = true;

        let mut merged_text = first.text.clone();
        let mut merged_x = first.x;
        let mut merged_y = first.y;
        let mut merged_right = first.x + first.width;
        let mut merged_bottom = first.y + first.height;

        let mut merged_line_count = 1usize;
        while merged_line_count < MAX_MERGED_LINES {
            let merged_height = merged_bottom - merged_y;
            let mut next_line_index = None;

            for next_index in (index + 1)..line_segments.len() {
                if consumed[next_index] {
                    continue;
                }

                let next = &line_segments[next_index];
                let next_bottom = next.y + next.height;
                let vertical_gap = horizontal_gap(merged_y, merged_bottom, next.y, next_bottom);
                if vertical_gap > merged_height.max(next.height) * MERGE_LINE_VERTICAL_FACTOR {
                    if next.y >= merged_bottom {
                        break;
                    }
                    continue;
                }

                if can_merge_multiline_segment(merged_x, merged_right, merged_height, next) {
                    next_line_index = Some(next_index);
                    break;
                }
            }

            let Some(next_index) = next_line_index else {
                break;
            };

            let next = &line_segments[next_index];
            consumed[next_index] = true;
            merged_text.push(' ');
            merged_text.push_str(&next.text);
            merged_x = merged_x.min(next.x);
            merged_y = merged_y.min(next.y);
            merged_right = merged_right.max(next.x + next.width);
            merged_bottom = merged_bottom.max(next.y + next.height);
            merged_line_count += 1;
        }

        merged_blocks.push(OcrWord {
            text: merged_text,
            x: merged_x,
            y: merged_y,
            width: merged_right - merged_x,
            height: merged_bottom - merged_y,
            slug: None,
            mapping_confidence: None,
            market_median: None,
            market_median_from_current_offers: None,
            ducats: None,
            vaulted: None,
            is_custom: None,
            trades_24h: None,
            moving_avg: None,
        });
    }

    merged_blocks
}
