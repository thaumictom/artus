use serde::Serialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize)]
pub struct UpdateAvailablePayload {
    pub version: String,
}

#[tauri::command]
pub async fn check_for_update<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<UpdateAvailablePayload>, String> {
    let updater = app
        .updater()
        .map_err(|err| format!("failed to create updater instance: {err}"))?;

    let Some(update) = updater
        .check()
        .await
        .map_err(|err| format!("failed to check for updates: {err}"))?
    else {
        println!("[updater] no update available");
        return Ok(None);
    };

    let version = update.version.to_string();
    println!("[updater] found update {version}");

    Ok(Some(UpdateAvailablePayload { version }))
}

#[tauri::command]
pub async fn download_and_relaunch_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let updater = app
        .updater()
        .map_err(|err| format!("failed to create updater instance: {err}"))?;

    let Some(update) = updater
        .check()
        .await
        .map_err(|err| format!("failed to check for updates: {err}"))?
    else {
        return Err("no update available".to_string());
    };

    println!("[updater] installing update {}", update.version);

    let mut downloaded = 0;
    update
        .download_and_install(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                println!("[updater] downloaded {downloaded} from {content_length:?}");
            },
            || {
                println!("[updater] download finished");
            },
        )
        .await
        .map_err(|err| format!("failed to download/install update: {err}"))?;

    println!("[updater] update installed, restarting app");
    app.restart();
}
