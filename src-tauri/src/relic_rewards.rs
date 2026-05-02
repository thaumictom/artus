//! Warframe EE.log tailer for automatic relic reward detection.
//!
//! Spawns an async task that polls the log file for reward markers
//! and triggers OCR or overlay hide accordingly.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::Ordering;
use std::time::Duration;

use log::{error, info};
use tauri::{AppHandle, Manager, Runtime};

use crate::ocr;
use crate::store_ext::SettingsExt;

// ── Log markers ───────────────────────────────────────────────────────────────

const GOT_REWARDS_MARKER: &str = "ProjectionRewardChoice.lua: Got rewards";
const SCREEN_SHUTDOWN_MARKER: &str = "ProjectionRewardChoice.lua: Relic reward screen shut down";

// ── Polling intervals ─────────────────────────────────────────────────────────

const DISABLED_SLEEP: Duration = Duration::from_millis(1000);
const IDLE_SLEEP: Duration = Duration::from_millis(100);
const ERROR_SLEEP: Duration = Duration::from_millis(500);

// ── Public API ────────────────────────────────────────────────────────────────

/// Spawns an async task that continuously tails the Warframe log file.
///
/// Uses `tokio::time::sleep` for non-blocking delays. File I/O is brief
/// (small incremental reads) so it runs on the async runtime directly.
pub fn spawn_log_tailer<R: Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut last_path = String::new();
        let mut last_pos: u64 = 0;
        let mut file: Option<File> = None;

        loop {
            let (enabled, path) = read_tailer_config(&app);

            if !enabled || path.is_empty() {
                reset_state(&mut file, &mut last_path, &mut last_pos);
                tokio::time::sleep(DISABLED_SLEEP).await;
                continue;
            }

            // Re-open on path change
            if path != last_path {
                open_log_file(&path, &mut file, &mut last_path, &mut last_pos);
            }

            // Read any new bytes appended since the last poll
            let data = read_new_data(&mut file, &last_path, &mut last_pos);

            if !data.is_empty() {
                process_log_chunk(&app, &data);
            }

            tokio::time::sleep(IDLE_SLEEP).await;
        }
    });
}

// ── Internals ─────────────────────────────────────────────────────────────────

/// Reads the current relic-reward settings from the store.
fn read_tailer_config<R: Runtime>(app: &AppHandle<R>) -> (bool, String) {
    let enabled = app.get_setting_bool("relic_reward_detection", false);
    let path = app.get_setting_string("warframe_log_path", "");
    (enabled, path)
}

/// Resets all tailer state when the feature is disabled.
fn reset_state(file: &mut Option<File>, path: &mut String, pos: &mut u64) {
    if file.is_some() {
        *file = None;
        path.clear();
        *pos = 0;
    }
}

/// Opens a new log file and seeks to the end (only processes new data).
fn open_log_file(path: &str, file: &mut Option<File>, last_path: &mut String, last_pos: &mut u64) {
    *last_path = path.to_string();
    *file = File::open(last_path.as_str()).ok();
    *last_pos = file
        .as_mut()
        .and_then(|f| f.seek(SeekFrom::End(0)).ok())
        .unwrap_or(0);

    if file.is_some() {
        info!("tailing log: {last_path}, starting at {last_pos} bytes");
    }
}

/// Reads bytes appended since `last_pos`. Handles file truncation/rotation.
fn read_new_data(file: &mut Option<File>, last_path: &str, last_pos: &mut u64) -> Vec<u8> {
    let mut buf = Vec::new();

    if let Some(f) = file.as_mut() {
        if let Ok(current_len) = f.seek(SeekFrom::End(0)) {
            if current_len > *last_pos {
                if f.seek(SeekFrom::Start(*last_pos)).is_ok() && f.read_to_end(&mut buf).is_ok() {
                    *last_pos = current_len;
                }
            } else if current_len < *last_pos {
                // File was truncated or rotated
                *last_pos = current_len;
            }
        }
    } else if !last_path.is_empty() {
        // Try to re-open after a brief delay (file may have been temporarily unavailable)
        std::thread::sleep(ERROR_SLEEP);
        *file = File::open(last_path).ok();
        *last_pos = file
            .as_mut()
            .and_then(|f| f.seek(SeekFrom::End(0)).ok())
            .unwrap_or(0);
    }

    buf
}

/// Inspects a chunk of log data for relic reward markers and acts accordingly.
fn process_log_chunk<R: Runtime>(app: &AppHandle<R>, data: &[u8]) {
    let content = String::from_utf8_lossy(data);

    if content.contains(SCREEN_SHUTDOWN_MARKER) {
        info!("detected relic reward screen shutdown, hiding overlay");
        let _ = ocr::hide_overlay(app);
    } else if content.contains(GOT_REWARDS_MARKER) {
        info!("detected relic rewards, triggering OCR");
        app.state::<crate::state::AppState>()
            .overlay_is_relic_mode
            .store(true, Ordering::Release);

        let handle = app.clone();
        let sequence = ocr::bump_overlay_sequence(&handle).unwrap_or(0);

        let failsafe_handle = handle.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_secs(15)).await;

            let current = failsafe_handle
                .state::<crate::state::AppState>()
                .overlay_sequence
                .lock()
                .map(|v| *v)
                .unwrap_or(0);

            if current == sequence {
                info!("relic reward 15s failsafe triggered, hiding overlay");
                let _ = ocr::hide_overlay(&failsafe_handle);
            }
        });

        tauri::async_runtime::spawn_blocking(move || {
            if let Err(err) = ocr::capture_active_window_with_mode(&handle, false, false, Some(sequence)) {
                error!("relic reward OCR failed: {err}");
            }
        });
    }
}
