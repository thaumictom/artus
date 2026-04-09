use std::str::FromStr;

use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::ocr;
use crate::state::AppState;

#[tauri::command]
pub fn get_hotkey(state: State<'_, AppState>) -> Result<String, String> {
    state
        .hotkey
        .lock()
        .map(|hotkey| hotkey.clone())
        .map_err(|_| "failed to read hotkey".to_string())
}

#[tauri::command]
pub fn set_hotkey<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    hotkey: String,
) -> Result<(), String> {
    let normalized = normalize_hotkey(&hotkey)?;

    let mut current = state
        .hotkey
        .lock()
        .map_err(|_| "failed to update hotkey".to_string())?;

    if *current == normalized {
        return Ok(());
    }

    let old = current.clone();
    if app.global_shortcut().is_registered(old.as_str()) {
        app.global_shortcut()
            .unregister(old.as_str())
            .map_err(|err| format!("failed to unregister old shortcut: {err}"))?;
    }

    app.global_shortcut()
        .register(normalized.as_str())
        .map_err(|err| format!("failed to register shortcut: {err}"))?;

    *current = normalized;
    Ok(())
}

pub fn register_initial<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let shortcut = app
        .state::<AppState>()
        .hotkey
        .lock()
        .map_err(|_| "failed to read startup hotkey".to_string())?
        .clone();

    if let Err(err) = app.global_shortcut().register(shortcut.as_str()) {
        eprintln!(
            "[hotkeys] startup shortcut '{}' unavailable: {}",
            shortcut, err
        );
    }

    Ok(())
}

pub fn on_pressed<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut) {
    let configured = match app.state::<AppState>().hotkey.lock() {
        Ok(value) => value.clone(),
        Err(_) => return,
    };

    if shortcut.into_string() != configured {
        return;
    }

    let app_handle = app.clone();
    std::thread::spawn(move || {
        if let Err(err) = ocr::capture_active_window(&app_handle) {
            eprintln!("ocr failed: {err}");
        }
    });
}

fn normalize_hotkey(input: &str) -> Result<String, String> {
    let cleaned = input.trim();
    let shortcut = Shortcut::from_str(cleaned).map_err(|err| format!("invalid shortcut: {err}"))?;
    Ok(shortcut.into_string())
}
