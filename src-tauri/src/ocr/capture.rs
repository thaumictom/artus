//! Screen capture, Tesseract OCR, and overlay display pipeline.
//!
//! The main entry point is [`capture_active_window_with_mode`], which runs the
//! full pipeline: capture → preprocess → OCR → group → match → display.

use std::time::{Duration, Instant};

use kreuzberg_tesseract::{TessPageIteratorLevel, TessPageSegMode, TesseractAPI};
use log::{error, info};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Runtime, Size};
use xcap::Window;

use super::{
    apply_morphology, binary_target_filter, gray_to_png_bytes,
    map_words_to_dictionary, resolve_tessdata, OcrDebugImagePayload, OcrPayload, OcrTextPayload,
    OcrWord, DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED,
    DEFAULT_OCR_TARGET_RGB, DEFAULT_OVERLAY_DURATION_SECS, ENABLE_OCR_DICTIONARY_MAPPING,
    MAX_OCR_DICTIONARY_MATCH_THRESHOLD, MIN_OCR_DICTIONARY_MATCH_THRESHOLD, OCR_WHITELIST,
    PASS_IMAGE_TO_FRONTEND, PASS_TEXT_TO_FRONTEND,
};
use crate::error::{AppError, AppResult};
use crate::layer_shell;
use crate::state::AppState;
use crate::store_ext::SettingsExt;
use tauri_plugin_store::StoreExt;

// ── Captured window metadata ──────────────────────────────────────────────────

/// Geometry and pixel data from a screen capture.
struct CapturedWindow {
    id: u64,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    image: image::RgbaImage,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn capture_active_window<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    capture_active_window_with_mode(app, true, true, None)
}

/// Toggles the overlay: if visible, hides it; otherwise captures and shows it.
pub fn toggle_overlay_hotkey<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| AppError::WindowNotFound("overlay".into()))?;

    let is_visible = overlay
        .is_visible()
        .map_err(|err| AppError::msg(format!("failed to read overlay visibility: {err}")))?;

    if is_visible {
        let _ = hide_overlay(app);
        return Ok(());
    }

    // Not visible → capture and show without auto-hide (toggle mode)
    capture_active_window_with_mode(app, false, true, None)
}

pub fn capture_active_window_with_mode<R: Runtime>(
    app: &AppHandle<R>,
    should_auto_hide: bool,
    is_manual: bool,
    provided_sequence: Option<u64>,
) -> AppResult<()> {
    let total = Instant::now();
    let run_sequence = provided_sequence.unwrap_or_else(|| bump_overlay_sequence(app).unwrap_or(0));

    let capture = capture_warframe_window()?;

    // Immediately show the overlay in "processing" state so the user
    // gets feedback before the slow OCR pipeline runs.
    show_overlay_processing(app, &capture)?;

    let filtered = preprocess_capture(app, &capture, is_manual);
    
    let config = app
        .store(crate::store_ext::SETTINGS_STORE_PATH)
        .ok()
        .and_then(|s| s.get("ocr_pipeline_config"))
        .and_then(|v| serde_json::from_value::<crate::ocr::OcrPipelineConfig>(v).ok())
        .unwrap_or_default();

    emit_debug_image(app, &filtered, config.emit_debug_overlay);
    let bounding_boxes = crate::ocr::engine::segment_image(&filtered);
    let words = run_tesseract_parallel(app, &filtered, &bounding_boxes, &config)?;
    let blocks = postprocess_words(app, &words, &capture, is_manual, &config);

    let current_sequence = app
        .state::<AppState>()
        .overlay_sequence
        .lock()
        .map(|v| *v)
        .unwrap_or(0);

    if current_sequence != run_sequence {
        info!("OCR pipeline sequence changed (aborted or replaced). Aborting show");
        return Ok(());
    }

    let hide_on_focus_loss = app.get_setting_bool("hide_overlay_on_focus_loss", true);
    if is_manual
        && hide_on_focus_loss
        && !app
            .state::<AppState>()
            .warframe_focused
            .load(std::sync::atomic::Ordering::Acquire)
    {
        info!("Warframe lost focus during manual OCR pipeline, aborting show");
        let _ = hide_overlay(app);
        return Ok(());
    }

    show_overlay(app, &capture, &blocks)?;

    if should_auto_hide {
        schedule_auto_hide(app, run_sequence)?;
    }

    info!("total OCR pipeline: {:?}", total.elapsed());
    Ok(())
}

// ── Step 1: Screen capture ────────────────────────────────────────────────────

/// Finds the Warframe window (regardless of focus, as long as it's not minimized) and captures its contents.
fn capture_warframe_window() -> AppResult<CapturedWindow> {
    let t = Instant::now();

    let window = Window::all()
        .map_err(|err| AppError::msg(format!("failed to list windows: {err}")))?
        .into_iter()
        .find(|w| {
            let name = w.app_name().unwrap_or_default().to_lowercase();
            let title = w.title().unwrap_or_default().to_lowercase();
            (name.contains("warframe") || title.contains("warframe"))
                && !w.is_minimized().unwrap_or(false)
        })
        .ok_or_else(|| AppError::msg("no non-minimized Warframe window found"))?;

    let id = window
        .id()
        .map_err(|e| AppError::msg(format!("failed to get id: {e}")))? as u64;
    let x = window
        .x()
        .map_err(|e| AppError::msg(format!("failed to get x: {e}")))?;
    let y = window
        .y()
        .map_err(|e| AppError::msg(format!("failed to get y: {e}")))?;
    let width = window
        .width()
        .map_err(|e| AppError::msg(format!("failed to get width: {e}")))?;
    let height = window
        .height()
        .map_err(|e| AppError::msg(format!("failed to get height: {e}")))?;
    let image = window
        .capture_image()
        .map_err(|err| AppError::msg(format!("failed to capture active window: {err}")))?;

    info!("window discovery + capture: {:?}", t.elapsed());
    Ok(CapturedWindow {
        id,
        x,
        y,
        width,
        height,
        image,
    })
}

// ── Step 2: Preprocessing ─────────────────────────────────────────────────────

/// Applies binary color filtering using the user's selected theme color.
fn preprocess_capture<R: Runtime>(
    app: &AppHandle<R>,
    capture: &CapturedWindow,
    is_manual: bool,
) -> image::GrayImage {
    let t = Instant::now();

    let theme_name = app.get_setting_string("ocr_theme", "EQUINOX");
    let target_rgb = app
        .state::<AppState>()
        .ocr_theme_colors
        .lock()
        .ok()
        .and_then(|map| map.get(&theme_name).copied())
        .unwrap_or(DEFAULT_OCR_TARGET_RGB);

    let capture_mods = app.get_setting_bool("capture_mods", false);

    let mut targets = vec![target_rgb];
    if is_manual && capture_mods {
        targets.push(crate::ocr::preprocessing::MOD_COLOR_GOLD);
        targets.push(crate::ocr::preprocessing::MOD_COLOR_SILVER);
        targets.push(crate::ocr::preprocessing::MOD_COLOR_BRONZE);
        targets.push(crate::ocr::preprocessing::MOD_COLOR_ARCHON);
        targets.push(crate::ocr::preprocessing::MOD_COLOR_SPECIAL);
    }

    let mut filtered = binary_target_filter(&capture.image, &targets);
    apply_morphology(&mut filtered);

    info!("preprocess (binary filter + morphology): {:?}", t.elapsed());
    filtered
}

// ── Step 3: Debug image ───────────────────────────────────────────────────────

/// Optionally encodes and emits the filtered image to the dashboard for debugging.
fn emit_debug_image<R: Runtime>(app: &AppHandle<R>, filtered: &image::GrayImage, emit_debug_overlay: bool) {
    if !PASS_IMAGE_TO_FRONTEND || !emit_debug_overlay {
        return;
    }

    let t = Instant::now();
    match gray_to_png_bytes(filtered) {
        Ok(png_bytes) => {
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
            info!("debug image encode + emit: {:?}", t.elapsed());
        }
        Err(err) => error!("failed to encode debug image: {err}"),
    }
}

// ── Step 4: Tesseract OCR (Parallel) ──────────────────────────────────────────

/// Feeds each crop into a thread-local Tesseract instance in parallel using Rayon.
fn run_tesseract_parallel<R: Runtime>(
    app: &AppHandle<R>,
    filtered: &image::GrayImage,
    bounding_boxes: &[(u32, u32, u32, u32)],
    config: &crate::ocr::OcrPipelineConfig,
) -> AppResult<Vec<OcrWord>> {
    let t = Instant::now();
    let tessdata = resolve_tessdata(app)?;
    
    let min_density = config.min_pixel_density;
    let upscale_factor = config.upscale_factor;
    let page_seg_mode = config.page_seg_mode;
    
    use rayon::prelude::*;
    use std::cell::RefCell;
    
    thread_local! {
        static TESSERACT: RefCell<Option<TesseractAPI>> = RefCell::new(None);
    }
    
    let words: Vec<OcrWord> = bounding_boxes
        .par_iter()
        .flat_map(|&(x, y, w, h)| {
            // Triage: check pixel density (0 is text)
            let mut text_pixels = 0;
            for cy in y..y+h {
                for cx in x..x+w {
                    if filtered.get_pixel(cx, cy)[0] == 0 {
                        text_pixels += 1;
                    }
                }
            }
            if text_pixels < min_density {
                return vec![];
            }
            
            // Crop
            let crop = image::imageops::crop_imm(filtered, x, y, w, h).to_image();
            
            // Upscale (Nearest Neighbor)
            let upscaled = image::imageops::resize(
                &crop,
                w * upscale_factor,
                h * upscale_factor,
                image::imageops::FilterType::Nearest,
            );
            
            TESSERACT.with(|api_cell| {
                let mut api_opt = api_cell.borrow_mut();
                if api_opt.is_none() {
                    let api = TesseractAPI::new();
                    let _ = api.init(tessdata.to_str().unwrap(), "eng");
                    
                    let psm = match page_seg_mode {
                        6 => TessPageSegMode::PSM_SINGLE_BLOCK,
                        7 => TessPageSegMode::PSM_SINGLE_LINE,
                        8 => TessPageSegMode::PSM_SINGLE_WORD,
                        13 => TessPageSegMode::PSM_RAW_LINE,
                        _ => TessPageSegMode::PSM_SINGLE_COLUMN, // Default fallback (4)
                    };
                    let _ = api.set_page_seg_mode(psm);
                    let _ = api.set_variable("tessedit_char_whitelist", OCR_WHITELIST);
                    *api_opt = Some(api);
                }
                
                let api = api_opt.as_mut().unwrap();
                let _ = api.set_image(
                    upscaled.as_raw(),
                    upscaled.width() as i32,
                    upscaled.height() as i32,
                    1,
                    upscaled.width() as i32,
                );
                let _ = api.recognize();
                
                let mut local_words = Vec::new();
                if let Ok(iter) = api.get_iterator() {
                    loop {
                        let text: String = iter
                            .get_utf8_text(TessPageIteratorLevel::RIL_WORD)
                            .unwrap_or_default()
                            .trim()
                            .chars()
                            .filter(|ch| OCR_WHITELIST.contains(*ch))
                            .collect();

                        if !text.is_empty() {
                            if let Ok((left, top, right, bottom)) =
                                iter.get_bounding_box(TessPageIteratorLevel::RIL_WORD)
                            {
                                let conf = iter.confidence(TessPageIteratorLevel::RIL_WORD).unwrap_or(0.0);
                                
                                // Map back to global coordinates
                                let global_x = x as f64 + (left as f64 / upscale_factor as f64);
                                let global_y = y as f64 + (top as f64 / upscale_factor as f64);
                                let global_w = (right - left) as f64 / upscale_factor as f64;
                                let global_h = (bottom - top) as f64 / upscale_factor as f64;
                                
                                let mut word = OcrWord::new(
                                    text,
                                    global_x,
                                    global_y,
                                    global_w,
                                    global_h,
                                );
                                word.confidence = conf;
                                local_words.push(word);
                            }
                        }

                        match iter.next(TessPageIteratorLevel::RIL_WORD) {
                            Ok(true) => continue,
                            _ => break,
                        }
                    }
                }
                
                local_words
            })
        })
        .collect();

    info!(
        "tesseract parallel setup + recognize: {:?} ({} words from {} crops)",
        t.elapsed(),
        words.len(),
        bounding_boxes.len()
    );
    Ok(words)
}

// ── Step 5: Post-processing ───────────────────────────────────────────────────

fn postprocess_words<R: Runtime>(
    app: &AppHandle<R>,
    words: &[OcrWord],
    capture: &CapturedWindow,
    is_manual: bool,
    config: &crate::ocr::OcrPipelineConfig,
) -> Vec<OcrWord> {
    let t = Instant::now();

    let mapping_enabled = app.get_setting_bool(
        "ocr_dictionary_mapping_enabled",
        DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED,
    );
    let mapping_threshold = config.dice_sorensen_threshold.clamp(
        MIN_OCR_DICTIONARY_MATCH_THRESHOLD,
        MAX_OCR_DICTIONARY_MATCH_THRESHOLD,
    );

    let mut finalized = if ENABLE_OCR_DICTIONARY_MAPPING && mapping_enabled {
        map_words_to_dictionary(app, words, mapping_threshold)
    } else {
        words.to_vec()
    };

    let capture_mods = app.get_setting_bool("capture_mods", false);
    if is_manual && capture_mods {
        for word in &mut finalized {
            word.mod_type = crate::ocr::preprocessing::identify_mod_type(&capture.image, word);
        }
    }

    let mapped = finalized.iter().filter(|w| w.slug.is_some()).count();
    let dropped = words.len().saturating_sub(finalized.len());

    info!(
        "postprocess: {:?} ({} mapped, {} dropped, mapping={}, threshold={:.2})",
        t.elapsed(), mapped, dropped,
        ENABLE_OCR_DICTIONARY_MAPPING && mapping_enabled, mapping_threshold,
    );

    // Optionally emit plain text to the dashboard
    if PASS_TEXT_TO_FRONTEND {
        let text = finalized
            .iter()
            .map(|w| w.text.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");

        if let Some(dashboard) = app.get_webview_window("artus") {
            let _ = dashboard.emit("ocr_text_result", OcrTextPayload { text });
        }
    }

    finalized
}

// ── Step 6: Overlay display ───────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn set_overlay_owner<R: Runtime>(app: &AppHandle<R>, capture: &CapturedWindow) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        if let Ok(hwnd) = overlay.hwnd() {
            let overlay_hwnd = windows::Win32::Foundation::HWND(hwnd.0 as *mut core::ffi::c_void);
            let target_hwnd = capture.id as isize;
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    overlay_hwnd,
                    windows::Win32::UI::WindowsAndMessaging::GWLP_HWNDPARENT,
                    target_hwnd,
                );
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn set_overlay_owner<R: Runtime>(_app: &AppHandle<R>, _capture: &CapturedWindow) {}

/// Positions, resizes, shows the overlay, and makes it click-through.
/// Returns whether the Wayland layer-shell path was used.
fn position_and_show_overlay<R: Runtime>(
    app: &AppHandle<R>,
    capture: &CapturedWindow,
) -> AppResult<bool> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| AppError::WindowNotFound("overlay".into()))?;

    let used_layer_shell =
        layer_shell::set_overlay_geometry(&overlay, capture.x, capture.y).unwrap_or(false);

    if !used_layer_shell {
        overlay
            .set_position(Position::Physical(PhysicalPosition::new(
                capture.x, capture.y,
            )))
            .map_err(|err| AppError::msg(format!("failed to position overlay: {err}")))?;
    }

    overlay
        .set_size(Size::Physical(PhysicalSize::new(
            capture.width,
            capture.height,
        )))
        .map_err(|err| AppError::msg(format!("failed to resize overlay: {err}")))?;

    set_overlay_owner(app, capture);

    overlay
        .show()
        .map_err(|err| AppError::msg(format!("failed to show overlay: {err}")))?;

    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = overlay.hwnd() {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                windows::Win32::Foundation::HWND(hwnd.0 as *mut core::ffi::c_void),
                Some(windows::Win32::Foundation::HWND(std::ptr::null_mut())), // HWND_TOP
                0,
                0,
                0,
                0,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            );
        }
    }

    if !layer_shell::is_wayland_session() || used_layer_shell {
        let _ = overlay.set_ignore_cursor_events(true);
        let _ = overlay.set_focusable(false);
    }

    if app
        .state::<AppState>()
        .warframe_focused
        .load(std::sync::atomic::Ordering::Acquire)
    {
        crate::hotkeys::register_escape_hotkey(app);
    }

    Ok(used_layer_shell)
}

/// Shows the overlay immediately (before OCR) and emits `ocr_processing` so
/// the frontend can render a "Processing…" spinner.
fn show_overlay_processing<R: Runtime>(
    app: &AppHandle<R>,
    capture: &CapturedWindow,
) -> AppResult<()> {
    position_and_show_overlay(app, capture)?;

    app.emit("ocr_processing", ())
        .map_err(|err| AppError::msg(format!("failed to emit ocr_processing: {err}")))?;

    Ok(())
}

/// Repositions the overlay with OCR results and emits them to the frontend.
/// Also applies the Wayland delayed click-through retry.
fn show_overlay<R: Runtime>(
    app: &AppHandle<R>,
    capture: &CapturedWindow,
    words: &[OcrWord],
) -> AppResult<()> {
    let t = Instant::now();

    position_and_show_overlay(app, capture)?;

    let force_click = app
        .get_webview_window("overlay")
        .and_then(|w| layer_shell::force_click_through(&w).ok())
        .unwrap_or(false);

    if layer_shell::is_wayland_session() && !force_click {
        error!("click-through not applied on initial show");
    }

    // Wayland: retry click-through after a short delay (compositor race)
    if layer_shell::is_wayland_session() {
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(60)).await;
            if let Some(retry_overlay) = app_clone.get_webview_window("overlay") {
                let _ = retry_overlay.set_ignore_cursor_events(true);
                let _ = retry_overlay.set_focusable(false);
                if !layer_shell::force_click_through(&retry_overlay).unwrap_or(false) {
                    error!("click-through not applied on delayed retry");
                }
            }
        });
    }

    // Emit OCR results to the frontend
    let show_bounding_boxes = app.get_setting_bool("show_ocr_bounding_boxes", false);

    app.emit(
        "ocr_result",
        OcrPayload {
            words: words.to_vec(),
            show_ocr_bounding_boxes: show_bounding_boxes,
        },
    )
    .map_err(|err| AppError::msg(format!("failed to emit OCR result: {err}")))?;

    info!("overlay show + emit: {:?}", t.elapsed());
    Ok(())
}

// ── Step 7: Auto-hide timer ───────────────────────────────────────────────────

/// Bumps the overlay sequence counter and schedules hiding after the configured
/// duration. If another capture occurs before the timer fires, the stale
/// sequence number prevents the hide.
fn schedule_auto_hide<R: Runtime>(app: &AppHandle<R>, sequence: u64) -> AppResult<()> {
    let duration_secs = app.get_setting_u64("overlay_duration_secs", DEFAULT_OVERLAY_DURATION_SECS);
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(duration_secs)).await;

        // Only hide if no newer capture has occurred
        let current = app_clone
            .state::<AppState>()
            .overlay_sequence
            .lock()
            .map(|v| *v)
            .unwrap_or(0);

        if current == sequence {
            let _ = hide_overlay(&app_clone);
        }
    });

    Ok(())
}

/// Increments and returns the overlay sequence counter.
pub fn bump_overlay_sequence<R: Runtime>(app: &AppHandle<R>) -> AppResult<u64> {
    let state = app.state::<AppState>();
    let mut guard = state.overlay_sequence.lock()?;
    *guard += 1;
    Ok(*guard)
}

/// Programmatically hides the overlay, resets state, and unregisters dynamic hotkeys.
pub fn hide_overlay<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    // Bump sequence to cancel any pending auto-hide/failsafes
    let _ = bump_overlay_sequence(app);

    app.state::<AppState>()
        .overlay_is_relic_mode
        .store(false, std::sync::atomic::Ordering::Release);

    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = app.emit("ocr_clear", ());
        crate::hotkeys::unregister_escape_hotkey(app);

        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = overlay.hide();
        });
    }

    Ok(())
}
