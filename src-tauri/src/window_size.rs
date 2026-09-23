//! Persist the main window's normal size without affecting the overlay window.

use serde::Deserialize;
use tauri::{App, LogicalSize, Manager, Size, Window, WindowEvent};
use tauri_plugin_store::StoreExt;

use crate::state::AppState;

const STORE_PATH: &str = "window-state.json";
const MIN_WIDTH: f64 = 550.0;
const MIN_HEIGHT: f64 = 350.0;

#[derive(Deserialize)]
struct SavedSize {
    width: f64,
    height: f64,
}

pub fn restore_artus_size(app: &App) {
    let Some(window) = app.get_webview_window("artus") else {
        return;
    };
    let saved = app
        .store(STORE_PATH)
        .ok()
        .and_then(|store| store.get("size"))
        .and_then(|value| serde_json::from_value::<SavedSize>(value).ok())
        .filter(|size| size.width.is_finite() && size.height.is_finite());
    if let Some(size) = saved {
        let width = size.width.max(MIN_WIDTH);
        let height = size.height.max(MIN_HEIGHT);
        if let Ok(mut normal_size) = app.state::<AppState>().artus_window_size.lock() {
            *normal_size = (width, height);
        }
        if let Err(error) = window.set_size(Size::Logical(LogicalSize::new(width, height))) {
            log::warn!("could not restore Artus window size: {error}");
        }
    }
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != "artus" {
        return;
    }
    match event {
        WindowEvent::Resized(size) => {
            if size.width == 0
                || size.height == 0
                || window.is_maximized().unwrap_or(false)
                || window.is_minimized().unwrap_or(false)
            {
                return;
            }
            if let Ok(scale) = window.scale_factor() {
                let logical = size.to_logical::<f64>(scale);
                if let Ok(mut normal_size) = window.state::<AppState>().artus_window_size.lock() {
                    *normal_size = (logical.width.max(MIN_WIDTH), logical.height.max(MIN_HEIGHT));
                }
            }
        }
        WindowEvent::CloseRequested { .. } => {
            if !window.is_maximized().unwrap_or(false) && !window.is_minimized().unwrap_or(false) {
                if let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) {
                    let logical = size.to_logical::<f64>(scale);
                    if let Ok(mut normal_size) = window.state::<AppState>().artus_window_size.lock()
                    {
                        *normal_size =
                            (logical.width.max(MIN_WIDTH), logical.height.max(MIN_HEIGHT));
                    }
                }
            }
            let size = window
                .state::<AppState>()
                .artus_window_size
                .lock()
                .ok()
                .map(|size| *size);
            if let Some((width, height)) = size {
                if let Ok(store) = window.app_handle().store(STORE_PATH) {
                    store.set(
                        "size",
                        serde_json::json!({ "width": width, "height": height }),
                    );
                    if let Err(error) = store.save() {
                        log::warn!("could not save Artus window size: {error}");
                    }
                }
            }
        }
        _ => {}
    }
}
