//! Windows relic selection sampling. The game window is read only.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use image::{DynamicImage, GrayImage, ImageFormat, RgbaImage};
use log::{info, warn};
use serde::Serialize;
use serde_json::{json, Value};
use std::io::Cursor;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_store::StoreExt;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    GetWindowDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

use crate::ocr::preprocessing::binary_target_filter_with_tolerance;
use crate::ocr::{OcrWord, THEME_COLORS_TOML};
use crate::store_ext::SettingsExt;

const START_DELAY: Duration = Duration::from_secs(10);
const SAMPLE_INTERVAL: Duration = Duration::from_millis(100);
const MAX_SAMPLE_AGE: Duration = Duration::from_secs(2);
const HIGHLIGHT_COLOR_TOLERANCE: u8 = 8;

#[derive(Clone, Copy)]
struct SelectionCluster {
    x: f64,
}

struct Session {
    sequence: u64,
    rewards: Vec<OcrWord>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    ocr_width: u32,
    ocr_word_count: usize,
    last_selection: Option<(OcrWord, Instant)>,
}

static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();
static LAST_CAPTURE: OnceLock<Mutex<Option<LastCapture>>> = OnceLock::new();

struct LastCapture {
    sequence: u64,
    image: GrayImage,
    selected_slot: Option<usize>,
    cluster_x: Option<u32>,
    matched_item: Option<String>,
    reward_count: usize,
    ocr_word_count: usize,
    reward_edges: Vec<(String, f64)>,
    status: String,
}

#[derive(Serialize)]
pub struct RelicSelectionDebugImage {
    png_bytes: Vec<u8>,
    width: u32,
    height: u32,
    selected_slot: Option<usize>,
    cluster_x: Option<u32>,
    matched_item: Option<String>,
    reward_count: usize,
    ocr_word_count: usize,
    reward_edges: Vec<(String, f64)>,
    status: String,
}

#[tauri::command]
pub fn get_relic_selection_debug_image() -> Result<Option<RelicSelectionDebugImage>, String> {
    let guard = LAST_CAPTURE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| "relic selection image lock poisoned".to_string())?;
    let Some(capture) = guard.as_ref() else {
        return Ok(None);
    };
    let mut bytes = Cursor::new(Vec::new());
    DynamicImage::ImageLuma8(capture.image.clone())
        .write_to(&mut bytes, ImageFormat::Png)
        .map_err(|err| format!("failed to encode relic selection image: {err}"))?;
    Ok(Some(RelicSelectionDebugImage {
        png_bytes: bytes.into_inner(),
        width: capture.image.width(),
        height: capture.image.height(),
        selected_slot: capture.selected_slot,
        cluster_x: capture.cluster_x,
        matched_item: capture.matched_item.clone(),
        reward_count: capture.reward_count,
        ocr_word_count: capture.ocr_word_count,
        reward_edges: capture.reward_edges.clone(),
        status: capture.status.clone(),
    }))
}

fn session() -> &'static Mutex<Option<Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
}

pub fn begin<R: Runtime + 'static>(app: &AppHandle<R>, sequence: u64) {
    if let Ok(mut guard) = session().lock() {
        *guard = Some(Session {
            sequence,
            rewards: Vec::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            ocr_width: 0,
            ocr_word_count: 0,
            last_selection: None,
        });
    }
    let app = app.clone();
    std::thread::spawn(move || {
        // DBWIN closes the session; this longer deadline only bounds a missing close event.
        let deadline = Instant::now() + Duration::from_secs(30);
        std::thread::sleep(START_DELAY);
        loop {
            if Instant::now() >= deadline
                || !app.get_setting_bool("relic_reward_detection", false)
                || !app.get_setting_bool("relic_reward_auto_add", true)
            {
                break;
            }
            let geometry = session().lock().ok().and_then(|guard| {
                guard.as_ref().and_then(|current| {
                    (current.sequence == sequence).then_some((
                        current.x,
                        current.y,
                        current.width,
                        current.height,
                    ))
                })
            });
            let Some((x, y, width, height)) = geometry else {
                break;
            };
            if width == 0 {
                std::thread::sleep(SAMPLE_INTERVAL);
                continue;
            }
            if let Ok(Some(image)) = capture_strip(x, y, width, height) {
                let Some(color) = highlight_color(&app) else {
                    break;
                };
                let filtered = binary_target_filter_with_tolerance(
                    &image,
                    &[color],
                    HIGHLIGHT_COLOR_TOLERANCE,
                );
                let cluster = selection_cluster(&filtered);
                if let Ok(mut guard) = session().lock() {
                    if let Some(current) = guard.as_mut().filter(|s| s.sequence == sequence) {
                        let matched = match_reward_by_right_edge(
                            cluster,
                            &current.rewards,
                            current.width,
                            current.ocr_width,
                        );
                        if let Ok(mut last) = LAST_CAPTURE.get_or_init(|| Mutex::new(None)).lock() {
                            *last = Some(LastCapture {
                                sequence,
                                image: filtered,
                                selected_slot: matched,
                                cluster_x: cluster.map(|cluster| cluster.x.round() as u32),
                                matched_item: matched
                                    .map(|index| current.rewards[index].text.clone()),
                                reward_count: current.rewards.len(),
                                ocr_word_count: current.ocr_word_count,
                                reward_edges: current
                                    .rewards
                                    .iter()
                                    .filter_map(|word| {
                                        strip_right_edge(word, current.width, current.ocr_width)
                                            .map(|edge| (word.text.clone(), edge))
                                    })
                                    .collect(),
                                status: "Sampling reward selection".into(),
                            });
                        }
                        if let Some(index) = matched {
                            current.last_selection =
                                Some((current.rewards[index].clone(), Instant::now()));
                        }
                    }
                }
            }
            std::thread::sleep(SAMPLE_INTERVAL);
        }
    });
}

pub fn record_rewards(
    sequence: u64,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    ocr_width: u32,
    words: &[OcrWord],
) {
    let rewards = cache_recognized_rewards(words);
    info!(
        "relic reward cache: {} mapped items from {} OCR words",
        rewards.len(),
        words.len()
    );
    if let Ok(mut guard) = session().lock() {
        if let Some(current) = guard.as_mut().filter(|s| s.sequence == sequence) {
            current.rewards = rewards;
            current.ocr_word_count = words.len();
            current.x = x;
            current.y = y;
            current.width = width;
            current.height = height;
            current.ocr_width = ocr_width;
        } else {
            warn!("discarded relic OCR cache for inactive sequence {sequence}");
        }
    }
}

fn cache_recognized_rewards(words: &[OcrWord]) -> Vec<OcrWord> {
    let mut rewards = words
        .iter()
        .filter(|word| word.slug.is_some())
        .cloned()
        .collect::<Vec<_>>();
    rewards.sort_by(|left, right| (left.x + left.width).total_cmp(&(right.x + right.width)));
    rewards
}

pub fn finish<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let current = session().lock().ok().and_then(|mut guard| guard.take());
    let Some(current) = current else { return None };
    let sequence = current.sequence;
    let mut added_name = None;
    let result = if !app.get_setting_bool("relic_reward_auto_add", true) {
        "Auto-add disabled".to_string()
    } else if let Some(word) = resolve_selection(current) {
        match add_reward_to_inventory(app, &word) {
            Ok(()) => {
                info!("relic auto-add saved: {}", word.text);
                added_name = Some(word.text.clone());
                format!("Added +1 {} to inventory", word.text)
            }
            Err(err) => {
                warn!("failed to add selected relic reward: {err}");
                format!("Inventory save failed: {err}")
            }
        }
    } else {
        "No recent reward matched the filtered selection".to_string()
    };
    if let Ok(mut guard) = LAST_CAPTURE.get_or_init(|| Mutex::new(None)).lock() {
        if let Some(capture) = guard
            .as_mut()
            .filter(|capture| capture.sequence == sequence)
        {
            capture.status = result;
        }
    }
    let _ = app.emit("relic_selection_capture_ready", ());
    added_name
}

pub fn show_added_feedback<R: Runtime + 'static>(app: &AppHandle<R>, name: String) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        let _ = app.emit("relic_reward_added", json!({ "name": name }));
    });
}

fn add_reward_to_inventory<R: Runtime>(app: &AppHandle<R>, word: &OcrWord) -> Result<(), String> {
    let store = app.store("inventory.json").map_err(|err| err.to_string())?;
    let mut items: Vec<Value> = store
        .get("items")
        .map(serde_json::from_value)
        .transpose()
        .map_err(|err| err.to_string())?
        .unwrap_or_default();
    let mut new_slugs: Vec<String> = store
        .get("newSlugs")
        .map(serde_json::from_value)
        .transpose()
        .map_err(|err| err.to_string())?
        .unwrap_or_default();
    apply_reward_to_items(&mut items, &mut new_slugs, word)?;
    store.set("items", json!(items));
    store.set("newSlugs", json!(new_slugs));
    store.save().map_err(|err| err.to_string())
}

fn apply_reward_to_items(
    items: &mut Vec<Value>,
    new_slugs: &mut Vec<String>,
    word: &OcrWord,
) -> Result<(), String> {
    let slug = word
        .slug
        .as_deref()
        .ok_or("selected reward has no item slug")?;
    let name_key = |name: &str| {
        name.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let existing = items.iter_mut().find(|item| {
        item.get("slug").and_then(Value::as_str).map_or_else(
            || {
                item.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name_key(name) == name_key(&word.text))
            },
            |saved_slug| saved_slug == slug,
        )
    });
    if let Some(existing) = existing {
        let quantity = existing
            .get("quantity")
            .and_then(Value::as_u64)
            .ok_or("existing inventory quantity is invalid")?;
        let next = quantity
            .checked_add(1)
            .ok_or("inventory quantity overflow")?;
        let object = existing
            .as_object_mut()
            .ok_or("existing inventory item is invalid")?;
        object.insert("quantity".into(), json!(next));
        object.entry("slug").or_insert_with(|| json!(slug));
        if let Some(price) = word.market_median.filter(|price| price.is_finite()) {
            object.insert("marketMedian".into(), json!(price));
            if let Some(fallback) = word.market_median_from_current_offers {
                object.insert("marketMedianUsesOfferFallback".into(), json!(fallback));
            }
        }
        if let Some(ducats) = word.ducats {
            object.entry("ducats").or_insert_with(|| json!(ducats));
        }
    } else {
        let mut item = json!({ "name": word.text, "slug": slug, "quantity": 1 });
        let object = item
            .as_object_mut()
            .expect("new inventory item is an object");
        if let Some(custom) = word.is_custom {
            object.insert("isCustom".into(), json!(custom));
        }
        if let Some(price) = word.market_median.filter(|price| price.is_finite()) {
            object.insert("marketMedian".into(), json!(price));
            if let Some(fallback) = word.market_median_from_current_offers {
                object.insert("marketMedianUsesOfferFallback".into(), json!(fallback));
            }
        }
        if let Some(ducats) = word.ducats {
            object.insert("ducats".into(), json!(ducats));
        }
        items.push(item);
        if !new_slugs.iter().any(|saved| saved == slug) {
            new_slugs.push(slug.to_string());
        }
    }
    Ok(())
}

fn resolve_selection(current: Session) -> Option<OcrWord> {
    let (word, time) = current.last_selection?;
    if time.elapsed() > MAX_SAMPLE_AGE {
        return None;
    }
    Some(word)
}

fn highlight_color<R: Runtime>(app: &AppHandle<R>) -> Option<[u8; 3]> {
    let theme = app.get_setting_string("ocr_theme", "EQUINOX");
    let parsed: toml::Value = toml::from_str(THEME_COLORS_TOML).ok()?;
    let values = parsed.get("highlight")?.get(&theme)?.as_array()?;
    Some([
        values.first()?.as_integer()?.try_into().ok()?,
        values.get(1)?.as_integer()?.try_into().ok()?,
        values.get(2)?.as_integer()?.try_into().ok()?,
    ])
}

fn selection_cluster(filtered: &GrayImage) -> Option<SelectionCluster> {
    if filtered.width() == 0 {
        return None;
    }
    let width = filtered.width() as usize;
    let mut columns = vec![0usize; width];
    for (index, pixel) in filtered.as_raw().iter().enumerate() {
        if *pixel == 0 {
            columns[index % width] += 1;
        }
    }
    let mut groups = Vec::new();
    let mut group: Option<(usize, usize, usize, usize)> = None; // start, last, pixels, weighted x
    for (x, count) in columns
        .into_iter()
        .enumerate()
        .filter(|(_, count)| *count > 0)
    {
        if let Some((start, last, pixels, weighted_x)) = group {
            if x > last + 3 {
                groups.push((start, last, pixels, weighted_x));
                group = Some((x, x, count, x * count));
            } else {
                group = Some((start, x, pixels + count, weighted_x + x * count));
            }
        } else {
            group = Some((x, x, count, x * count));
        }
    }
    if let Some(group) = group {
        groups.push(group);
    }
    groups.sort_by_key(|(_, _, pixels, _)| std::cmp::Reverse(*pixels));
    let &(start, end, pixels, weighted_x) = groups.first()?;
    let runner_up = groups.get(1).map(|group| group.2).unwrap_or(0);
    if pixels < 8 || end - start < 2 || pixels < runner_up.saturating_mul(2) {
        return None;
    }
    Some(SelectionCluster {
        x: weighted_x as f64 / pixels as f64,
    })
}

/// Compare the cluster with each cached OCR box's right edge in strip coordinates.
fn match_reward_by_right_edge(
    cluster: Option<SelectionCluster>,
    rewards: &[OcrWord],
    window_width: u32,
    ocr_width: u32,
) -> Option<usize> {
    let cluster_x = cluster?.x;
    rewards
        .iter()
        .enumerate()
        .filter_map(|(index, word)| {
            strip_right_edge(word, window_width, ocr_width)
                .map(|edge| (index, (edge - cluster_x).abs()))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(index, _)| index)
}

fn strip_right_edge(word: &OcrWord, window_width: u32, ocr_width: u32) -> Option<f64> {
    if window_width == 0 || ocr_width == 0 {
        return None;
    }
    let edge = word.x + word.width;
    if !edge.is_finite() {
        return None;
    }
    Some(edge * window_width as f64 / ocr_width as f64 - (window_width / 20) as f64)
}

fn capture_strip(x: i32, y: i32, width: u32, height: u32) -> Result<Option<RgbaImage>, String> {
    if width < 320 || height < 240 {
        return Ok(None);
    }
    let left = width as i32 / 20;
    let top = height as i32 * 20 / 100;
    let strip_width = width as i32 * 9 / 10;
    let strip_height = (height as i32 * 6 / 100).max(8);
    let hwnd: HWND = unsafe { GetDesktopWindow() };
    unsafe {
        let source = GetWindowDC(Some(hwnd));
        if source.0.is_null() {
            return Err("GetWindowDC failed".into());
        }
        let memory = CreateCompatibleDC(Some(source));
        let bitmap = CreateCompatibleBitmap(source, strip_width, strip_height);
        if memory.0.is_null() || bitmap.0.is_null() {
            if !bitmap.0.is_null() {
                let _ = DeleteObject(bitmap.into());
            }
            if !memory.0.is_null() {
                let _ = DeleteDC(memory);
            }
            ReleaseDC(Some(hwnd), source);
            return Err("GDI strip allocation failed".into());
        }
        let old = SelectObject(memory, bitmap.into());
        let copied = BitBlt(
            memory,
            0,
            0,
            strip_width,
            strip_height,
            Some(source),
            x + left,
            y + top,
            SRCCOPY,
        );
        let mut pixels = vec![0u8; (strip_width * strip_height * 4) as usize];
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: strip_width,
                biHeight: -strip_height,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        let rows = if copied.is_ok() {
            GetDIBits(
                memory,
                bitmap,
                0,
                strip_height as u32,
                Some(pixels.as_mut_ptr().cast()),
                &mut info,
                DIB_RGB_COLORS,
            )
        } else {
            0
        };
        SelectObject(memory, old);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory);
        ReleaseDC(Some(hwnd), source);
        if rows == 0 {
            return Ok(None);
        }
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        Ok(RgbaImage::from_raw(
            strip_width as u32,
            strip_height as u32,
            pixels,
        ))
    }
}
