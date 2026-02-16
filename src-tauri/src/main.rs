#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use image::{DynamicImage, RgbImage};
use kreuzberg_tesseract::{TessPageIteratorLevel, TesseractAPI};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

const ENABLE_BINARY_PREVIEW: bool = false;
const ENABLE_DILATION: bool = false;
const OCR_CHAR_WHITELIST: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789- ";

// State struct to hold the initialized OCR Engine
struct OcrState {
    engine: Mutex<TesseractAPI>,
}

#[derive(Serialize, Clone)]
struct OcrWordBox {
    text: String,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Serialize, Clone)]
struct OcrOverlayData {
    window_left: i32,
    window_top: i32,
    words: Vec<OcrWordBox>,
}

fn sanitize_ocr_text(input: &str) -> String {
    let filtered: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect();

    filtered.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn should_merge_words(previous: &OcrWordBox, current: &OcrWordBox) -> bool {
    let previous_height = (previous.bottom - previous.top).max(1) as f32;
    let current_height = (current.bottom - current.top).max(1) as f32;

    let previous_center_y = (previous.top + previous.bottom) as f32 / 2.0;
    let current_center_y = (current.top + current.bottom) as f32 / 2.0;
    let center_delta = (previous_center_y - current_center_y).abs();
    let max_center_delta = previous_height.max(current_height) * 0.55;
    if center_delta > max_center_delta {
        return false;
    }

    let gap = (current.left - previous.right) as f32;
    if gap < -2.0 {
        return false;
    }

    let prev_width = (previous.right - previous.left).max(1) as f32;
    let prev_len = previous.text.len().max(1) as f32;
    let estimated_char_width = (prev_width / prev_len).max(1.0);
    let max_join_gap = (estimated_char_width * 2.8).max(previous_height * 0.9);

    gap <= max_join_gap
}

fn merge_word_boxes(words: Vec<OcrWordBox>) -> Vec<OcrWordBox> {
    let mut merged = Vec::new();

    for word in words {
        if let Some(last) = merged.last_mut() {
            if should_merge_words(last, &word) {
                last.text.push(' ');
                last.text.push_str(&word.text);
                last.left = last.left.min(word.left);
                last.top = last.top.min(word.top);
                last.right = last.right.max(word.right);
                last.bottom = last.bottom.max(word.bottom);
                continue;
            }
        }
        merged.push(word);
    }

    merged
}

fn capture_active_window() -> Option<DynamicImage> {
    // 1. Identify the active window details
    let active_win = active_win_pos_rs::get_active_window().ok()?;

    // 2. Fetch all open windows from xcap
    let windows = xcap::Window::all().ok()?;

    // 3. Match the active window to an xcap window (heuristic by title and app name)
    let target = windows
        .iter()
        .find(|w| {
            w.title().unwrap_or_default() == active_win.title
                && w.app_name().unwrap_or_default() == active_win.app_name
        })
        .or_else(|| {
            windows
                .iter()
                .find(|w| w.title().unwrap_or_default() == active_win.title)
        })?;

    // 4. Capture the pixel data
    let image = target.capture_image().ok()?;

    Some(DynamicImage::ImageRgba8(image.into()))
}

fn apply_binary_filter(img: DynamicImage) -> RgbImage {
    let mut luma_img = img.into_luma8();
    for pixel in luma_img.pixels_mut() {
        pixel[0] = if pixel[0] > 238 { 255 } else { 0 };
    }
    DynamicImage::ImageLuma8(luma_img).into_rgb8()
}

// Special function that makes every pixel white, except for #242424
fn apply_aggressive_binary_filter(img: DynamicImage) -> RgbImage {
    let mut rgb_img = img.into_rgb8();
    for pixel in rgb_img.pixels_mut() {
        if pixel[0] == 0x24 && pixel[1] == 0x24 && pixel[2] == 0x24 {
            *pixel = image::Rgb([0, 0, 0]);
        } else {
            *pixel = image::Rgb([255, 255, 255]);
        }
    }
    rgb_img
}

fn preprocess_for_ocr(img: DynamicImage) -> RgbImage {
    let filtered = apply_aggressive_binary_filter(img);
    if ENABLE_DILATION {
        dilate_image(&filtered)
    } else {
        filtered
    }
}

// Dilation function to thicken the black pixels
fn dilate_image(img: &RgbImage) -> RgbImage {
    let (width, height) = img.dimensions();
    let mut dilated = img.clone();

    for y in 0..height {
        for x in 0..width {
            if img.get_pixel(x, y)[0] == 0 {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            dilated.put_pixel(nx as u32, ny as u32, image::Rgb([0, 0, 0]));
                        }
                    }
                }
            }
        }
    }

    dilated
}

// 3. Do OCR
fn perform_ocr(
    engine: &TesseractAPI,
    binary_img: RgbImage,
) -> Result<(String, Vec<OcrWordBox>), Box<dyn std::error::Error>> {
    let (width, height) = binary_img.dimensions();
    let bytes_per_pixel: i32 = 3;
    let bytes_per_line = (width as i32) * bytes_per_pixel;

    engine.set_image(
        binary_img.as_raw(),
        width as i32,
        height as i32,
        bytes_per_pixel,
        bytes_per_line,
    )?;
    let extracted_text = sanitize_ocr_text(&engine.get_utf8_text()?);

    let mut words = Vec::new();
    if let Ok(iterator) = engine.get_iterator() {
        loop {
            let text = iterator
                .get_utf8_text(TessPageIteratorLevel::RIL_WORD)
                .unwrap_or_default();
            let sanitized_text = sanitize_ocr_text(&text);

            if !sanitized_text.is_empty() {
                if let Ok((left, top, right, bottom)) =
                    iterator.get_bounding_box(TessPageIteratorLevel::RIL_WORD)
                {
                    words.push(OcrWordBox {
                        text: sanitized_text,
                        left,
                        top,
                        right,
                        bottom,
                    });
                }
            }

            if !iterator
                .next(TessPageIteratorLevel::RIL_WORD)
                .unwrap_or(false)
            {
                break;
            }
        }
    }

    println!("Extracted Text:\n{}", extracted_text); // Debug print to console

    let merged_words = merge_word_boxes(words);

    Ok((extracted_text, merged_words))
}

#[derive(Serialize, Clone)]
struct ImageData {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

// Coordinator
fn process_and_emit(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();
    let capture_start = Instant::now();
    let active_win = active_win_pos_rs::get_active_window()
        .map_err(|_| std::io::Error::other("Failed to get active window position"))?;
    let raw_img = capture_active_window().ok_or("Failed to capture active window")?;
    let capture_elapsed = capture_start.elapsed();

    let preprocess_start = Instant::now();
    let binary_img = preprocess_for_ocr(raw_img);
    let preprocess_elapsed = preprocess_start.elapsed();

    if ENABLE_BINARY_PREVIEW {
        // Emit binary image
        let (width, height) = binary_img.dimensions();
        let image_data = ImageData {
            width,
            height,
            data: binary_img.clone().into_raw(),
        };

        if let Some(dashboard_window) = app.get_webview_window("dashboard") {
            dashboard_window.emit("binary-image", image_data)?;
        }
    }

    // Fetch state and lock mutex
    let state: State<OcrState> = app.state();
    let engine = state.engine.lock().unwrap();

    let ocr_start = Instant::now();
    let (text, words) = perform_ocr(&engine, binary_img)?;
    let ocr_elapsed = ocr_start.elapsed();

    if let Some(overlay_window) = app.get_webview_window("overlay") {
        let overlay_data = OcrOverlayData {
            window_left: active_win.position.x.round() as i32,
            window_top: active_win.position.y.round() as i32,
            words,
        };
        overlay_window.emit("ocr-overlay", overlay_data)?;
    }

    app.emit("ocr-result", text)?;
    println!(
        "OCR timings: capture={:?}, preprocess={:?}, ocr={:?}, total={:?}",
        capture_elapsed,
        preprocess_elapsed,
        ocr_elapsed,
        total_start.elapsed()
    );
    Ok(())
}

fn main() {
    let engine = TesseractAPI::new();
    engine
        .init("tessdata", "eng")
        .expect("Failed to initialize Tesseract engine");
    let _ = engine.set_variable("load_system_dawg", "0");
    let _ = engine.set_variable("load_freq_dawg", "0");
    let _ = engine.set_variable("tessedit_char_whitelist", OCR_CHAR_WHITELIST);
    let _ = engine.set_variable("preserve_interword_spaces", "1");
    let _ = engine.set_variable("tessedit_pageseg_mode", "11");

    let ocr_state = OcrState {
        engine: Mutex::new(engine),
    };

    println!("OCR Engine initialized successfully.");

    tauri::Builder::default()
        // Manage state
        .manage(ocr_state)
        .setup(|app| {
            // Ensure overlay remains borderless and click-through
            let window = app.get_webview_window("overlay").unwrap();
            let _ = window.set_fullscreen(false);
            window.set_decorations(false).unwrap();
            #[cfg(target_os = "windows")]
            {
                let _ = window.set_shadow(false);
            }
            if let Ok(Some(monitor)) = window.current_monitor() {
                let monitor_size = monitor.size();
                let monitor_position = monitor.position();
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
                    monitor_size.width,
                    monitor_size.height,
                )));
                let _ = window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition::new(monitor_position.x, monitor_position.y),
                ));
            }
            window.set_ignore_cursor_events(true).unwrap();

            let app_handle = app.handle().clone();
            let start_ocr_shortcut = Shortcut::new(None, Code::Home);

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &start_ocr_shortcut
                            && event.state() == ShortcutState::Pressed
                        {
                            // Print to console for debugging
                            println!("Shortcut triggered: {:?}", shortcut);

                            let handle = app_handle.clone();
                            std::thread::spawn(move || {
                                if let Err(e) = process_and_emit(handle) {
                                    eprintln!("OCR Process Error: {}", e);
                                }
                            });
                        }
                    })
                    .build(),
            )?;

            app.global_shortcut().register(start_ocr_shortcut)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
