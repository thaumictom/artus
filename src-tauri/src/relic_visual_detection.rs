//! Experimental visual relic-reward detector for Windows and Linux.
//!
//! This module is intentionally self-contained so it can be removed without
//! affecting normal OCR or the Windows DBWIN listener.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use image::imageops::{crop_imm, resize, FilterType};
use kreuzberg_tesseract::{TessPageIteratorLevel, TessPageSegMode, TesseractAPI};
use log::{error, info, warn};
use tauri::{AppHandle, Manager, Runtime};
use xcap::Window;

use crate::error::{AppError, AppResult};
use crate::ocr::{
    apply_morphology, binary_target_filter, group_words, resolve_tessdata, OcrWord,
    DEFAULT_OCR_TARGET_RGB, OCR_WHITELIST,
};
use crate::relic_reward_capture;
use crate::state::AppState;
use crate::store_ext::SettingsExt;

const SETTING_KEY: &str = "visual_relic_reward_detection";
const SCAN_INTERVAL: Duration = Duration::from_millis(300);
const DISABLED_INTERVAL: Duration = Duration::from_millis(500);
const RETRY_INTERVAL: Duration = Duration::from_secs(5);
const COOLDOWN: Duration = Duration::from_secs(2);
const CONFIRM_FRAMES: u8 = 2;
const DISMISS_FRAMES: u8 = 3;
const DETECTOR_MATCH_THRESHOLD: f64 = 0.82;
const UPSCALE_FACTOR: u32 = 2;

fn visual_detection_enabled<R: Runtime>(app: &AppHandle<R>) -> bool {
    if !app.get_setting_bool(SETTING_KEY, false) {
        return false;
    }

    // DBWIN wins if an old or manually edited Windows settings file contains
    // both flags. The settings UI normally prevents this state.
    #[cfg(target_os = "windows")]
    if app.get_setting_bool("relic_reward_detection", false) {
        return false;
    }

    true
}

struct VisualMatch {
    signature: String,
}

struct RewardCandidate {
    slug: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Default)]
struct DetectorState {
    pending_signature: Option<String>,
    matching_frames: u8,
    missing_frames: u8,
    reward_visible: bool,
    owns_overlay: bool,
    cooldown_until: Option<Instant>,
}

impl DetectorState {
    fn reset_pending(&mut self) {
        self.pending_signature = None;
        self.matching_frames = 0;
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

pub fn spawn_visual_listener<R: Runtime + 'static>(app: AppHandle<R>) {
    if let Err(err) = std::thread::Builder::new()
        .name("warframe-visual-relic-detector".into())
        .spawn(move || run(app))
    {
        error!("failed to spawn visual relic reward detector: {err}");
    }
}

fn run<R: Runtime + 'static>(app: AppHandle<R>) {
    let mut detector = DetectorState::default();
    let mut api: Option<TesseractAPI> = None;

    loop {
        if !visual_detection_enabled(&app) {
            if detector.owns_overlay {
                let _ = crate::ocr::hide_overlay(&app);
            }
            detector.reset();
            api = None;
            std::thread::sleep(DISABLED_INTERVAL);
            continue;
        }

        if !app
            .state::<AppState>()
            .warframe_focused
            .load(Ordering::Acquire)
        {
            detector.reset_pending();
            std::thread::sleep(SCAN_INTERVAL);
            continue;
        }

        if api.is_none() {
            match create_tesseract(&app) {
                Ok(created) => api = Some(created),
                Err(err) => {
                    warn!("visual relic detector unavailable: {err}");
                    std::thread::sleep(RETRY_INTERVAL);
                    continue;
                }
            }
        }

        let visual_match = match detect_reward_row(&app, api.as_ref().expect("initialized above")) {
            Ok(result) => result,
            Err(err) => {
                warn!("visual relic detector scan failed: {err}");
                api = None;
                std::thread::sleep(RETRY_INTERVAL);
                continue;
            }
        };

        update_detector_state(&app, &mut detector, visual_match);
        std::thread::sleep(SCAN_INTERVAL);
    }
}

fn create_tesseract<R: Runtime>(app: &AppHandle<R>) -> AppResult<TesseractAPI> {
    let tessdata = resolve_tessdata(app)?;
    let api = TesseractAPI::new();
    api.init(&tessdata, "eng")
        .map_err(|err| AppError::msg(format!("failed to init detector tesseract: {err}")))?;
    api.set_page_seg_mode(TessPageSegMode::PSM_SPARSE_TEXT)
        .map_err(|err| AppError::msg(format!("failed to configure detector tesseract: {err}")))?;
    api.set_variable("tessedit_char_whitelist", OCR_WHITELIST)
        .map_err(|err| AppError::msg(format!("failed to configure detector whitelist: {err}")))?;
    Ok(api)
}

fn detect_reward_row<R: Runtime>(
    app: &AppHandle<R>,
    api: &TesseractAPI,
) -> AppResult<Option<VisualMatch>> {
    let image = capture_warframe_window()?;
    let width = image.width();
    let height = image.height();
    if width < 320 || height < 240 {
        return Ok(None);
    }

    // Broad enough to include the header and reward names, while excluding most
    // HUD text and the lower endless-mission controls.
    let crop_x = width / 20;
    let crop_y = height * 12 / 100;
    let crop_width = width * 9 / 10;
    let crop_height = height * 66 / 100;
    let reward_region = crop_imm(&image, crop_x, crop_y, crop_width, crop_height).to_image();

    let theme_name = app.get_setting_string("ocr_theme", "EQUINOX");
    let target_rgb = app
        .state::<AppState>()
        .ocr_theme_colors
        .lock()
        .ok()
        .and_then(|colors| colors.get(&theme_name).copied())
        .unwrap_or(DEFAULT_OCR_TARGET_RGB);

    let mut filtered = binary_target_filter(&reward_region, &[target_rgb]);
    let foreground_pixels = filtered
        .as_raw()
        .iter()
        .filter(|pixel| **pixel == 0)
        .count();
    let foreground_ratio = foreground_pixels as f64 / filtered.as_raw().len() as f64;
    if !(0.0002..=0.12).contains(&foreground_ratio) {
        return Ok(None);
    }

    apply_morphology(&mut filtered);
    let upscaled = resize(
        &filtered,
        filtered.width() * UPSCALE_FACTOR,
        filtered.height() * UPSCALE_FACTOR,
        FilterType::Nearest,
    );
    let words = recognize_words(api, &upscaled)?;
    classify_reward_row(app, words, crop_width as f64, crop_height as f64)
}

fn capture_warframe_window() -> AppResult<image::RgbaImage> {
    let window = Window::all()
        .map_err(|err| AppError::msg(format!("failed to list windows: {err}")))?
        .into_iter()
        .find(|window| {
            let app_name = window.app_name().unwrap_or_default().to_lowercase();
            let title = window.title().unwrap_or_default().to_lowercase();
            (app_name.contains("warframe") || title.contains("warframe"))
                && !window.is_minimized().unwrap_or(false)
        })
        .ok_or_else(|| AppError::msg("no non-minimized Warframe window found"))?;

    window
        .capture_image()
        .map_err(|err| AppError::msg(format!("failed to capture Warframe window: {err}")))
}

fn recognize_words(api: &TesseractAPI, image: &image::GrayImage) -> AppResult<Vec<OcrWord>> {
    api.set_image(
        image.as_raw(),
        image.width() as i32,
        image.height() as i32,
        1,
        image.width() as i32,
    )
    .map_err(|err| AppError::msg(format!("failed to set detector image: {err}")))?;
    api.recognize()
        .map_err(|err| AppError::msg(format!("detector recognition failed: {err}")))?;
    let iterator = api
        .get_iterator()
        .map_err(|err| AppError::msg(format!("failed to read detector OCR: {err}")))?;

    let mut words = Vec::new();
    loop {
        let text: String = iterator
            .get_utf8_text(TessPageIteratorLevel::RIL_WORD)
            .unwrap_or_default()
            .trim()
            .chars()
            .filter(|character| OCR_WHITELIST.contains(*character))
            .collect();

        if !text.is_empty() {
            if let Ok((left, top, right, bottom)) =
                iterator.get_bounding_box(TessPageIteratorLevel::RIL_WORD)
            {
                words.push(OcrWord::new(
                    text,
                    left as f64 / UPSCALE_FACTOR as f64,
                    top as f64 / UPSCALE_FACTOR as f64,
                    (right - left) as f64 / UPSCALE_FACTOR as f64,
                    (bottom - top) as f64 / UPSCALE_FACTOR as f64,
                ));
            }
        }

        match iterator.next(TessPageIteratorLevel::RIL_WORD) {
            Ok(true) => continue,
            _ => break,
        }
    }

    Ok(words)
}

fn classify_reward_row<R: Runtime>(
    app: &AppHandle<R>,
    words: Vec<OcrWord>,
    region_width: f64,
    region_height: f64,
) -> AppResult<Option<VisualMatch>> {
    if words.is_empty() {
        return Ok(None);
    }

    let raw_text = words
        .iter()
        .map(|word| word.text.to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");
    let has_reward_header =
        (raw_text.contains("SELECT") || raw_text.contains("CHOOSE")) && raw_text.contains("REWARD");

    let grouped = group_words(app, words);
    let mut candidates = match_reward_candidates(app, &grouped)?
        .into_iter()
        .filter(|word| {
            let center_y = word.y + word.height / 2.0;
            center_y >= region_height * 0.18 && center_y <= region_height * 0.88
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() || candidates.len() > 4 {
        return Ok(None);
    }

    candidates.sort_by(|left, right| {
        let left_center = left.x + left.width / 2.0;
        let right_center = right.x + right.width / 2.0;
        left_center.total_cmp(&right_center)
    });

    let center_ys = candidates
        .iter()
        .map(|word| word.y + word.height / 2.0)
        .collect::<Vec<_>>();
    let average_y = center_ys.iter().sum::<f64>() / center_ys.len() as f64;
    let baseline_tolerance = region_height * 0.06;
    if center_ys
        .iter()
        .any(|center_y| (center_y - average_y).abs() > baseline_tolerance)
    {
        return Ok(None);
    }

    let count = candidates.len();
    let centers = candidates
        .iter()
        .map(|word| (word.x + word.width / 2.0) / region_width)
        .collect::<Vec<_>>();
    let slots_match = centers.iter().enumerate().all(|(index, center)| {
        let expected = (index + 1) as f64 / (count + 1) as f64;
        (center - expected).abs() <= 0.13
    });
    if !slots_match {
        return Ok(None);
    }

    // A lone dictionary match is common in other menus. Require the explicit
    // reward header as an additional independent signal for solo fissures.
    if count == 1 && !has_reward_header {
        return Ok(None);
    }

    let signature = candidates
        .iter()
        .map(|word| word.slug.as_str())
        .collect::<Vec<_>>()
        .join("|");
    if signature.is_empty() {
        return Ok(None);
    }

    Ok(Some(VisualMatch { signature }))
}

/// Performs detection-only dictionary matching. Unlike the full OCR mapper,
/// this never loads prices or performs network I/O inside the scan loop.
fn match_reward_candidates<R: Runtime>(
    app: &AppHandle<R>,
    words: &[OcrWord],
) -> AppResult<Vec<RewardCandidate>> {
    let state = app.state::<AppState>();
    let dictionary = state
        .ocr_dictionary
        .lock()
        .map_err(|_| AppError::msg("OCR dictionary lock poisoned"))?;
    if dictionary.is_empty() {
        return Ok(Vec::new());
    }

    Ok(words
        .iter()
        .filter_map(|word| {
            let normalized = crate::ocr::dictionary::normalize_dictionary_text(&word.text);
            if normalized.is_empty() {
                return None;
            }

            let (entry, score) = dictionary
                .iter()
                .filter(|entry| !entry.is_relic)
                .map(|entry| (entry, similarity_score(&normalized, &entry.normalized_name)))
                .max_by(|left, right| left.1.total_cmp(&right.1))?;

            (score >= DETECTOR_MATCH_THRESHOLD).then(|| RewardCandidate {
                slug: entry.slug.clone(),
                x: word.x,
                y: word.y,
                width: word.width,
                height: word.height,
            })
        })
        .collect())
}

fn similarity_score(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }
    let max_len = left.len().max(right.len());
    if max_len == 0 {
        return 0.0;
    }

    let edit_score =
        1.0 - levenshtein_distance(left.as_bytes(), right.as_bytes()) as f64 / max_len as f64;
    let left_tokens = left.split_whitespace().collect::<Vec<_>>();
    let right_tokens = right.split_whitespace().collect::<Vec<_>>();
    let shared = left_tokens
        .iter()
        .filter(|token| right_tokens.contains(token))
        .count();
    let token_score = shared as f64 / left_tokens.len().max(right_tokens.len()).max(1) as f64;
    (edit_score * 0.85 + token_score * 0.15).clamp(0.0, 1.0)
}

fn levenshtein_distance(left: &[u8], right: &[u8]) -> usize {
    if left.is_empty() {
        return right.len();
    }
    if right.is_empty() {
        return left.len();
    }

    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_byte) in left.iter().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_byte) in right.iter().enumerate() {
            let substitution = usize::from(left_byte != right_byte);
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

fn update_detector_state<R: Runtime>(
    app: &AppHandle<R>,
    state: &mut DetectorState,
    visual_match: Option<VisualMatch>,
) {
    if state.reward_visible {
        // Another detector may have owned the shared overlay and then stopped
        // before this detector did. Re-arm while the visual match is still
        // present so the enabled visual detector can take over.
        let shared_capture_active = app
            .state::<AppState>()
            .overlay_is_relic_mode
            .load(Ordering::Acquire);
        if !state.owns_overlay && !shared_capture_active && visual_match.is_some() {
            state.reward_visible = false;
            state.reset_pending();
        } else {
            if visual_match.is_some() {
                state.missing_frames = 0;
            } else {
                state.missing_frames = state.missing_frames.saturating_add(1);
                if state.missing_frames >= DISMISS_FRAMES {
                    if state.owns_overlay {
                        info!("visual relic reward screen disappeared, hiding overlay");
                        let _ = crate::ocr::hide_overlay(app);
                    }
                    state.reset();
                    state.cooldown_until = Some(Instant::now() + COOLDOWN);
                }
            }
            return;
        }
    }

    if state
        .cooldown_until
        .is_some_and(|deadline| Instant::now() < deadline)
    {
        return;
    }
    state.cooldown_until = None;

    let Some(visual_match) = visual_match else {
        state.reset_pending();
        return;
    };

    if state.pending_signature.as_deref() == Some(&visual_match.signature) {
        state.matching_frames = state.matching_frames.saturating_add(1);
    } else {
        state.pending_signature = Some(visual_match.signature);
        state.matching_frames = 1;
    }

    if state.matching_frames >= CONFIRM_FRAMES {
        state.owns_overlay =
            relic_reward_capture::trigger(app, SETTING_KEY, "visual detector", Duration::ZERO);
        state.reward_visible = true;
        state.missing_frames = 0;
        state.reset_pending();
    }
}
