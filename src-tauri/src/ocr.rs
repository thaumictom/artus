use std::env;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use image::{DynamicImage, GrayImage, ImageFormat};
use imageproc::distance_transform::Norm;
use imageproc::morphology::{dilate_mut, erode_mut};
use kreuzberg_tesseract::{TessPageIteratorLevel, TessPageSegMode, TesseractAPI};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Runtime, Size};
use xcap::Window;

use crate::dictionary;
use crate::state::AppState;

const OCR_WHITELIST: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789- ";
const STRICT_NAMES: bool = true;
const STRICT_NAME_SCORE_THRESHOLD: f64 = 0.72;
const PASS_IMAGE_TO_FRONTEND: bool = true;
const PASS_TEXT_TO_FRONTEND: bool = false;
const TARGET_R: u8 = 158;
const TARGET_G: u8 = 159;
const TARGET_B: u8 = 167;
const ENABLE_MORPHOLOGY: bool = false;
pub const BINARY_FILTER_SPILL_THRESHOLD: u8 = 0;
pub const HORIZONTAL_WORD_GAP_FACTOR: f64 = 1.2;
pub const SAME_LINE_VERTICAL_FACTOR: f64 = 0.5;
pub const MERGE_LINE_VERTICAL_FACTOR: f64 = 1.0;
pub const MAX_MERGED_LINES: usize = 2;

#[derive(Debug, Clone, Serialize)]
pub struct OcrWord {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrPayload {
    pub words: Vec<OcrWord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrDebugImagePayload {
    pub png_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub upscale_amount: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrTextPayload {
    pub text: String,
}

#[derive(Debug, Clone)]
struct RawWord {
    text: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub fn capture_active_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let total_started = Instant::now();

    let discovery_started = Instant::now();
    let window = Window::all()
        .map_err(|err| format!("failed to list windows: {err}"))?
        .into_iter()
        .find(|entry| entry.is_focused().unwrap_or(false) && !entry.is_minimized().unwrap_or(false))
        .ok_or("no focused window found")?;

    let x = window
        .x()
        .map_err(|err| format!("failed to get x: {err}"))?;
    let y = window
        .y()
        .map_err(|err| format!("failed to get y: {err}"))?;
    let width = window
        .width()
        .map_err(|err| format!("failed to get width: {err}"))?;
    let height = window
        .height()
        .map_err(|err| format!("failed to get height: {err}"))?;

    let image = window
        .capture_image()
        .map_err(|err| format!("failed to capture active window: {err}"))?;
    println!(
        "[ocr] window discovery + capture: {:?}",
        discovery_started.elapsed()
    );

    let preprocess_started = Instant::now();
    let mut filtered = binary_target_filter(&image);
    apply_morphology(&mut filtered);

    println!(
        "[ocr] preprocess (binary filter + erosion): {:?}",
        preprocess_started.elapsed()
    );

    if PASS_IMAGE_TO_FRONTEND {
        let debug_started = Instant::now();
        let png_bytes = gray_to_png_bytes(&filtered)?;
        if let Some(dashboard) = app.get_webview_window("dashboard") {
            let _ = dashboard.emit(
                "ocr_debug_image",
                OcrDebugImagePayload {
                    png_bytes,
                    width: filtered.width(),
                    height: filtered.height(),
                    upscale_amount: 1,
                },
            );
        }
        println!(
            "[ocr] debug image encode + emit: {:?}",
            debug_started.elapsed()
        );
    }

    let ocr_started = Instant::now();
    let tessdata = resolve_tessdata(app)?;
    let api = TesseractAPI::new();
    api.init(&tessdata, "eng")
        .map_err(|err| format!("failed to init tesseract: {err}"))?;

    api.set_page_seg_mode(TessPageSegMode::PSM_SPARSE_TEXT)
        .map_err(|err| format!("failed to set page segmentation: {err}"))?;
    api.set_variable("tessedit_char_whitelist", OCR_WHITELIST)
        .map_err(|err| format!("failed to set whitelist: {err}"))?;
    api.set_image(
        filtered.as_raw(),
        filtered.width() as i32,
        filtered.height() as i32,
        1,
        filtered.width() as i32,
    )
    .map_err(|err| format!("failed to set image: {err}"))?;

    api.recognize()
        .map_err(|err| format!("recognition failed: {err}"))?;
    let iter = api
        .get_iterator()
        .map_err(|err| format!("failed to get OCR iterator: {err}"))?;
    println!(
        "[ocr] tesseract setup + recognize: {:?}",
        ocr_started.elapsed()
    );

    let parse_started = Instant::now();
    let mut words = Vec::new();
    loop {
        let text = iter
            .get_utf8_text(TessPageIteratorLevel::RIL_WORD)
            .unwrap_or_default()
            .trim()
            .chars()
            .filter(|ch| OCR_WHITELIST.contains(*ch))
            .collect::<String>();

        if !text.is_empty() {
            if let Ok((left, top, right, bottom)) =
                iter.get_bounding_box(TessPageIteratorLevel::RIL_WORD)
            {
                words.push(OcrWord {
                    text,
                    x: left as f64,
                    y: top as f64,
                    width: (right - left) as f64,
                    height: (bottom - top) as f64,
                });
            }
        }

        let has_next = iter
            .next(TessPageIteratorLevel::RIL_WORD)
            .map_err(|err| format!("iterator error: {err}"))?;
        if !has_next {
            break;
        }
    }

    let grouped_words = apply_strict_names(app, group_words_into_blocks(&words))?;
    println!(
        "[ocr] parse OCR words: {:?} ({} words -> {} blocks)",
        parse_started.elapsed(),
        words.len(),
        grouped_words.len()
    );

    if PASS_TEXT_TO_FRONTEND {
        let text_started = Instant::now();
        let text = grouped_words
            .iter()
            .map(|word| word.text.trim())
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");

        if let Some(dashboard) = app.get_webview_window("dashboard") {
            let _ = dashboard.emit("ocr_text_result", OcrTextPayload { text });
        }
        println!("[ocr] text emit: {:?}", text_started.elapsed());
    }

    let overlay_started = Instant::now();
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("overlay window not found")?;

    overlay
        .set_position(Position::Physical(PhysicalPosition::new(x, y)))
        .map_err(|err| format!("failed to position overlay: {err}"))?;
    overlay
        .set_size(Size::Physical(PhysicalSize::new(width, height)))
        .map_err(|err| format!("failed to resize overlay: {err}"))?;
    overlay
        .show()
        .map_err(|err| format!("failed to show overlay: {err}"))?;
    overlay
        .emit(
            "ocr_result",
            OcrPayload {
                words: grouped_words,
            },
        )
        .map_err(|err| format!("failed to emit OCR result: {err}"))?;
    println!("[ocr] overlay show + emit: {:?}", overlay_started.elapsed());

    let sequence = {
        let app_state = app.state::<AppState>();
        let mut guard = app_state
            .overlay_sequence
            .lock()
            .map_err(|_| "failed to update overlay sequence")?;
        *guard += 1;
        *guard
    };

    let app_handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(10));
        let current_sequence = app_handle
            .state::<AppState>()
            .overlay_sequence
            .lock()
            .map(|value| *value)
            .unwrap_or(0);
        if current_sequence != sequence {
            return;
        }

        if let Some(overlay) = app_handle.get_webview_window("overlay") {
            let _ = overlay.hide();
        }
    });

    println!("[ocr] total: {:?}", total_started.elapsed());

    Ok(())
}

fn binary_target_filter(source: &image::RgbaImage) -> GrayImage {
    let width = source.width() as usize;
    let height = source.height() as usize;
    let raw = source.as_raw();
    let mut output = vec![255u8; width * height];

    for y in 0..height {
        for x in 0..width {
            let src_idx = (y * width + x) * 4;
            let r = raw[src_idx];
            let g = raw[src_idx + 1];
            let b = raw[src_idx + 2];
            output[y * width + x] = if matches_target_color(r, g, b, TARGET_R, TARGET_G, TARGET_B)
            // || matches_target_color(r, g, b, TARGET_R_ALT, TARGET_G_ALT, TARGET_B_ALT)
            {
                0
            } else {
                255
            };
        }
    }

    GrayImage::from_raw(source.width(), source.height(), output)
        .expect("invalid binary filter output dimensions")
}

fn apply_morphology(source: &mut GrayImage) {
    if !ENABLE_MORPHOLOGY {
        return;
    }

    // Experimental
    erode_mut(source, Norm::L1, 1);
    dilate_mut(source, Norm::L1, 1);
}

fn matches_target_color(r: u8, g: u8, b: u8, target_r: u8, target_g: u8, target_b: u8) -> bool {
    r.abs_diff(target_r) <= BINARY_FILTER_SPILL_THRESHOLD
        && g.abs_diff(target_g) <= BINARY_FILTER_SPILL_THRESHOLD
        && b.abs_diff(target_b) <= BINARY_FILTER_SPILL_THRESHOLD
}

fn gray_to_png_bytes(gray: &GrayImage) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    DynamicImage::ImageLuma8(gray.clone())
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| format!("failed to encode debug image: {err}"))?;
    Ok(bytes)
}

fn apply_strict_names<R: Runtime>(
    app: &AppHandle<R>,
    words: Vec<OcrWord>,
) -> Result<Vec<OcrWord>, String> {
    if !STRICT_NAMES {
        return Ok(words);
    }

    let dictionary = load_strict_name_dictionary(app)?;
    if dictionary.is_empty() {
        return Ok(vec![]);
    }

    Ok(words
        .into_iter()
        .filter_map(|word| {
            match_closest_dictionary_name(&dictionary, &word.text)
                .map(|text| OcrWord { text, ..word })
        })
        .collect())
}

fn load_strict_name_dictionary<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<String>, String> {
    dictionary::load_cached_dictionary_names(app)
}

fn normalize_dictionary_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn match_closest_dictionary_name(dictionary: &[String], fragment: &str) -> Option<String> {
    let normalized_fragment = normalize_dictionary_name(fragment);
    if normalized_fragment.is_empty() {
        return None;
    }

    let mut best_name = None;
    let mut best_score = 0.0;

    for candidate in dictionary {
        let score = normalized_similarity(&normalized_fragment, candidate);
        if score > best_score {
            best_score = score;
            best_name = Some(candidate.clone());
        }
    }

    if best_score >= STRICT_NAME_SCORE_THRESHOLD {
        best_name
    } else {
        None
    }
}

fn normalized_similarity(left: &str, right: &str) -> f64 {
    let left = left.to_ascii_lowercase();
    let right = right.to_ascii_lowercase();

    if left == right {
        return 1.0;
    }

    let max_len = left.chars().count().max(right.chars().count());
    if max_len == 0 {
        return 1.0;
    }

    let distance = levenshtein_distance(&left, &right);
    1.0 - (distance as f64 / max_len as f64)
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();

    if left_chars.is_empty() {
        return right_chars.len();
    }
    if right_chars.is_empty() {
        return left_chars.len();
    }

    let mut previous_row = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current_row = vec![0; right_chars.len() + 1];

    for (left_index, left_char) in left_chars.iter().enumerate() {
        current_row[0] = left_index + 1;

        for (right_index, right_char) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_char != right_char);
            current_row[right_index + 1] = (current_row[right_index] + 1)
                .min(previous_row[right_index + 1] + 1)
                .min(previous_row[right_index] + substitution_cost);
        }

        previous_row.clone_from_slice(&current_row);
    }

    previous_row[right_chars.len()]
}

fn resolve_tessdata<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let mut candidates = vec![];

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("tessdata"));
    }

    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("src-tauri").join("tessdata"));
        candidates.push(cwd.join("tessdata"));
    }

    candidates
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| "could not find tessdata directory".to_string())
}

fn group_words_into_blocks(words: &[OcrWord]) -> Vec<OcrWord> {
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

        let mut second_line_index = None;
        for next_index in (index + 1)..line_segments.len() {
            if consumed[next_index] {
                continue;
            }

            let next = &line_segments[next_index];
            let vertical_gap = next.y - merged_bottom;
            if vertical_gap < 0.0 {
                continue;
            }
            if vertical_gap > first.height.max(next.height) * MERGE_LINE_VERTICAL_FACTOR {
                break;
            }

            let first_left = merged_x;
            let first_right = merged_right;
            let next_left = next.x;
            let next_right = next.x + next.width;
            let overlap = first_left.max(next_left) <= first_right.min(next_right);
            if overlap {
                second_line_index = Some(next_index);
                break;
            }
        }

        if let Some(next_index) = second_line_index {
            if MAX_MERGED_LINES >= 2 {
                let next = &line_segments[next_index];
                consumed[next_index] = true;
                merged_text.push('\n');
                merged_text.push_str(&next.text);
                merged_x = merged_x.min(next.x);
                merged_y = merged_y.min(next.y);
                merged_right = merged_right.max(next.x + next.width);
                merged_bottom = merged_bottom.max(next.y + next.height);
            }
        }

        merged_blocks.push(OcrWord {
            text: merged_text,
            x: merged_x,
            y: merged_y,
            width: merged_right - merged_x,
            height: merged_bottom - merged_y,
        });
    }

    merged_blocks
}
