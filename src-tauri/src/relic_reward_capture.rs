//! Shared relic-reward capture trigger used by platform-specific detectors.

use std::sync::atomic::Ordering;
use std::time::Duration;

use log::{error, info};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::ocr;
use crate::store_ext::SettingsExt;

/// Starts one relic OCR run if `enabled_setting` is still enabled and no relic
/// capture is already active. Returns whether this caller owns the new run.
pub fn trigger<R: Runtime>(
    app: &AppHandle<R>,
    enabled_setting: &'static str,
    source: &'static str,
    delay: Duration,
) -> bool {
    if !app.get_setting_bool(enabled_setting, false) {
        return false;
    }

    let state = app.state::<crate::state::AppState>();
    if state
        .overlay_is_relic_mode
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        info!("ignored duplicate relic reward trigger from {source}");
        return false;
    }

    info!("detected relic rewards via {source}, scheduling OCR after {delay:?}");
    if app.get_setting_bool("relic_reward_sound", false) {
        let _ = app.emit("relic_reward_detected", ());
    }

    let handle = app.clone();
    let sequence = match ocr::bump_overlay_sequence(&handle) {
        Ok(sequence) => sequence,
        Err(err) => {
            state.overlay_is_relic_mode.store(false, Ordering::Release);
            error!("failed to start relic reward capture sequence: {err}");
            return false;
        }
    };

    let failsafe_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;

        let current = failsafe_handle
            .state::<crate::state::AppState>()
            .overlay_sequence
            .lock()
            .map(|value| *value)
            .unwrap_or(0);

        if current == sequence {
            info!("relic reward 15s failsafe triggered, hiding overlay");
            let _ = ocr::hide_overlay(&failsafe_handle);
        }
    });

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(delay).await;

        if !handle.get_setting_bool(enabled_setting, false) {
            let _ = ocr::hide_overlay(&handle);
            return;
        }

        let current_sequence = handle
            .state::<crate::state::AppState>()
            .overlay_sequence
            .lock()
            .map(|value| *value)
            .unwrap_or(0);

        if current_sequence != sequence {
            info!("cancelled delayed relic reward OCR because the reward screen closed");
            return;
        }

        tauri::async_runtime::spawn_blocking(move || {
            if let Err(err) =
                ocr::capture_active_window_with_mode(&handle, false, false, Some(sequence), false)
            {
                error!("relic reward OCR failed: {err}");
                let _ = ocr::hide_overlay(&handle);
            }
        });
    });

    true
}
