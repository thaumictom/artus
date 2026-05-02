//! Automatic update checking and installation via tauri-plugin-updater.

use log::info;
use serde::Serialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_updater::UpdaterExt;

use crate::error::{AppError, AppResult};

/// Payload sent to the frontend when an update is available.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateAvailablePayload {
    pub version: String,
}

/// Checks for a new version. Returns `None` if already up to date.
#[tauri::command]
pub async fn check_for_update<R: Runtime>(
    app: AppHandle<R>,
) -> AppResult<Option<UpdateAvailablePayload>> {
    let updater = app
        .updater()
        .map_err(|err| AppError::msg(format!("failed to create updater: {err}")))?;

    let Some(update) = updater
        .check()
        .await
        .map_err(|err| AppError::msg(format!("failed to check for updates: {err}")))?
    else {
        info!("no update available");
        return Ok(None);
    };

    let version = update.version.to_string();
    info!("found update: {version}");
    Ok(Some(UpdateAvailablePayload { version }))
}

/// Downloads the pending update, installs it, and relaunches the application.
#[tauri::command]
pub async fn download_and_relaunch_update<R: Runtime>(app: AppHandle<R>) -> AppResult<()> {
    let updater = app
        .updater()
        .map_err(|err| AppError::msg(format!("failed to create updater: {err}")))?;

    let Some(update) = updater
        .check()
        .await
        .map_err(|err| AppError::msg(format!("failed to check for updates: {err}")))?
    else {
        return Err(AppError::msg("no update available"));
    };

    info!("installing update {}", update.version);

    let mut downloaded = 0;
    update
        .download_and_install(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                info!("downloaded {downloaded} from {content_length:?}");
            },
            || {
                info!("download finished");
            },
        )
        .await
        .map_err(|err| AppError::msg(format!("failed to download/install update: {err}")))?;

    info!("update installed, restarting app");
    app.restart();
}
