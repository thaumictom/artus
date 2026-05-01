use std::time::{Duration, Instant};

use kreuzberg_tesseract::{TessPageIteratorLevel, TessPageSegMode, TesseractAPI};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Runtime, Size};
use xcap::Window;

use crate::layer_shell;
use crate::state::AppState;
use super::{
    binary_target_filter, apply_morphology, gray_to_png_bytes, resolve_tessdata, group_words_into_blocks,
    map_words_to_dictionary,
    OcrWord, OcrPayload, OcrDebugImagePayload, OcrTextPayload,
    DEFAULT_OCR_TARGET_RGB, DEFAULT_OVERLAY_DURATION_SECS,
    DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED, DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD,
    ENABLE_OCR_DICTIONARY_MAPPING, MAX_OCR_DICTIONARY_MATCH_THRESHOLD, MIN_OCR_DICTIONARY_MATCH_THRESHOLD,
    OCR_WHITELIST, PASS_IMAGE_TO_FRONTEND, PASS_TEXT_TO_FRONTEND,
};

pub fn capture_active_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    capture_active_window_with_mode(app, true)
}

pub fn toggle_overlay_hotkey<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("overlay window not found")?;
    let is_visible = overlay
        .is_visible()
        .map_err(|err| format!("failed to read overlay visibility: {err}"))?;

    if is_visible {
        let _ = bump_overlay_sequence(app)?;
        overlay
            .hide()
            .map_err(|err| format!("failed to hide overlay: {err}"))?;
        return Ok(());
    }

    capture_active_window_with_mode(app, false)
}

pub fn bump_overlay_sequence<R: Runtime>(app: &AppHandle<R>) -> Result<u64, String> {
    let app_state = app.state::<AppState>();
    let mut guard = app_state
        .overlay_sequence
        .lock()
        .map_err(|_| "failed to update overlay sequence".to_string())?;
    *guard += 1;
    Ok(*guard)
}

pub fn capture_active_window_with_mode<R: Runtime>(
    app: &AppHandle<R>,
    should_auto_hide: bool,
) -> Result<(), String> {
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
    let target_rgb = app
        .state::<AppState>()
        .ocr_target_rgb
        .lock()
        .map(|value| *value)
        .unwrap_or(DEFAULT_OCR_TARGET_RGB);

    let mut filtered = binary_target_filter(&image, target_rgb);
    apply_morphology(&mut filtered);

    println!(
        "[ocr] preprocess (binary filter + erosion): {:?}",
        preprocess_started.elapsed()
    );

    if PASS_IMAGE_TO_FRONTEND {
        let debug_started = Instant::now();
        let png_bytes = gray_to_png_bytes(&filtered)?;
        if let Some(dashboard) = app.get_webview_window("artus") {
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
        }

        let has_next = iter
            .next(TessPageIteratorLevel::RIL_WORD)
            .map_err(|err| format!("iterator error: {err}"))?;
        if !has_next {
            break;
        }
    }

    let app_state = app.state::<AppState>();
    let mapping_enabled = app_state
        .ocr_dictionary_mapping_enabled
        .lock()
        .map(|value| *value)
        .unwrap_or(DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED);
    let mapping_threshold = app_state
        .ocr_dictionary_match_threshold
        .lock()
        .map(|value| *value)
        .unwrap_or(DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD)
        .clamp(
            MIN_OCR_DICTIONARY_MATCH_THRESHOLD,
            MAX_OCR_DICTIONARY_MATCH_THRESHOLD,
        );

    let grouped_words = group_words_into_blocks(&words);
    let finalized_words = if ENABLE_OCR_DICTIONARY_MAPPING && mapping_enabled {
        map_words_to_dictionary(app, &grouped_words, mapping_threshold)
    } else {
        grouped_words.clone()
    };
    let mapped_count = finalized_words
        .iter()
        .filter(|word| word.slug.is_some())
        .count();
    let dropped_count = grouped_words.len().saturating_sub(finalized_words.len());
    println!(
        "[ocr] parse OCR words: {:?} ({} words -> {} blocks, {} mapped, {} dropped, mapping {}, threshold {:.2})",
        parse_started.elapsed(),
        words.len(),
        grouped_words.len(),
        mapped_count,
        dropped_count,
        ENABLE_OCR_DICTIONARY_MAPPING && mapping_enabled,
        mapping_threshold,
    );

    if PASS_TEXT_TO_FRONTEND {
        let text_started = Instant::now();
        let text = finalized_words
            .iter()
            .map(|word| word.text.trim())
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");

        if let Some(dashboard) = app.get_webview_window("artus") {
            let _ = dashboard.emit("ocr_text_result", OcrTextPayload { text });
        }
        println!("[ocr] text emit: {:?}", text_started.elapsed());
    }

    let overlay_started = Instant::now();
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("overlay window not found")?;

    let used_layer_shell_positioning =
        layer_shell::set_overlay_geometry(&overlay, x, y).unwrap_or(false);

    if !used_layer_shell_positioning {
        overlay
            .set_position(Position::Physical(PhysicalPosition::new(x, y)))
            .map_err(|err| format!("failed to position overlay: {err}"))?;
    }

    overlay
        .set_size(Size::Physical(PhysicalSize::new(width, height)))
        .map_err(|err| format!("failed to resize overlay: {err}"))?;
    overlay
        .show()
        .map_err(|err| format!("failed to show overlay: {err}"))?;

    if !layer_shell::is_wayland_session() || used_layer_shell_positioning {
        let _ = overlay.set_ignore_cursor_events(true);
        let _ = overlay.set_focusable(false);
    }

    let force_click_applied = layer_shell::force_click_through(&overlay).unwrap_or(false);
    if layer_shell::is_wayland_session() && !force_click_applied {
        eprintln!("[overlay] click-through not applied on initial show");
    }

    if layer_shell::is_wayland_session() {
        let app_handle_retry = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            if let Some(overlay_retry) = app_handle_retry.get_webview_window("overlay") {
                let _ = overlay_retry.set_ignore_cursor_events(true);
                let _ = overlay_retry.set_focusable(false);
                let retry_applied =
                    layer_shell::force_click_through(&overlay_retry).unwrap_or(false);
                if !retry_applied {
                    eprintln!("[overlay] click-through not applied on delayed retry");
                }
            }
        });
    }

    let show_ocr_bounding_boxes = app
        .state::<AppState>()
        .show_ocr_bounding_boxes
        .lock()
        .map(|value| *value)
        .unwrap_or(false);

    app.emit(
        "ocr_result",
        OcrPayload {
            words: finalized_words,
            show_ocr_bounding_boxes,
        },
    )
    .map_err(|err| format!("failed to emit OCR result: {err}"))?;
    println!("[ocr] overlay show + emit: {:?}", overlay_started.elapsed());

    let sequence = bump_overlay_sequence(app)?;

    if should_auto_hide {
        let app_handle = app.clone();
        let overlay_duration_secs = app
            .state::<AppState>()
            .overlay_duration_secs
            .lock()
            .map(|value| *value)
            .unwrap_or(DEFAULT_OVERLAY_DURATION_SECS);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(overlay_duration_secs));
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
    }

    println!("[ocr] total: {:?}", total_started.elapsed());

    Ok(())
}
