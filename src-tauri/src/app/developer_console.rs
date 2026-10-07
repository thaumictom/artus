//! Opt-in webview developer tools, shared by the app and overlay windows.

use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::error::{AppError, AppResult};
use crate::store_ext::SETTINGS_STORE_PATH;

pub async fn apply(app: &AppHandle, enabled: bool) -> AppResult<()> {
    for window in app.webview_windows().into_values() {
        if !enabled {
            window.close_devtools();
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        window
            .with_webview(move |webview| {
                #[cfg(target_os = "windows")]
                let result = unsafe {
                    webview
                        .controller()
                        .CoreWebView2()
                        .and_then(|view| view.Settings())
                        .and_then(|settings| settings.SetAreDevToolsEnabled(enabled))
                        .map_err(AppError::msg)
                };
                #[cfg(target_os = "linux")]
                let result = {
                    use webkit2gtk::{SettingsExt, WebViewExt};
                    webview
                        .inner()
                        .settings()
                        .ok_or_else(|| AppError::msg("webview settings unavailable"))
                        .map(|settings| settings.set_enable_developer_extras(enabled))
                };
                let _ = sender.send(result);
            })
            .map_err(AppError::msg)?;
        receiver.await.map_err(AppError::msg)??;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_developer_console_enabled(app: AppHandle, enabled: bool) -> AppResult<()> {
    apply(&app, enabled).await?;
    let store = app.store(SETTINGS_STORE_PATH).map_err(AppError::msg)?;
    store.set("developer_console_enabled", serde_json::json!(enabled));
    store.save().map_err(AppError::msg)?;
    Ok(())
}
