use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_store::StoreExt;

use crate::ocr;
use crate::state::{AppState, DEFAULT_SCREENSHOT_HOTKEY, HOTKEY_ACTION_SCREENSHOT};

const SETTINGS_STORE_PATH: &str = "settings.json";
const HOTKEYS_STORE_KEY: &str = "hotkeys";
const HOTKEY_DEFINITIONS: [(&str, &str); 1] =
    [(HOTKEY_ACTION_SCREENSHOT, DEFAULT_SCREENSHOT_HOTKEY)];

#[tauri::command]
pub fn get_hotkey(state: State<'_, AppState>, action: String) -> Result<String, String> {
    let action_key = normalize_action(&action);
    ensure_known_action(&action_key)?;

    let hotkeys = state
        .hotkeys
        .lock()
        .map_err(|_| "failed to read hotkey".to_string())?;

    let stored = hotkeys
        .get(action_key.as_str())
        .cloned()
        .or_else(|| default_shortcut_for_action(&action_key).map(str::to_string))
        .ok_or_else(|| format!("missing hotkey for action: {action_key}"))?;

    Ok(format_hotkey_for_client(stored.as_str()))
}

#[tauri::command]
pub fn set_hotkey<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    action: String,
    hotkey: String,
) -> Result<String, String> {
    let action_key = normalize_action(&action);
    ensure_known_action(&action_key)?;

    let normalized = normalize_hotkey(&hotkey)?;

    let (current_shortcut, mut next_hotkeys) = {
        let hotkeys = state
            .hotkeys
            .lock()
            .map_err(|_| "failed to update hotkey".to_string())?;

        let current_shortcut = hotkeys
            .get(action_key.as_str())
            .cloned()
            .or_else(|| default_shortcut_for_action(&action_key).map(str::to_string))
            .ok_or_else(|| format!("missing hotkey for action: {action_key}"))?;

        if current_shortcut == normalized {
            return Ok(format_hotkey_for_client(normalized.as_str()));
        }

        if hotkeys.iter().any(|(other_action, other_shortcut)| {
            other_action != &action_key && other_shortcut == &normalized
        }) {
            return Err(format!(
                "hotkey '{normalized}' is already used by another action"
            ));
        }

        let mut cloned = hotkeys.clone();
        cloned.insert(action_key.clone(), normalized.clone());
        (current_shortcut, cloned)
    };

    app.global_shortcut()
        .register(normalized.as_str())
        .map_err(|err| format!("failed to register shortcut: {err}"))?;

    if app
        .global_shortcut()
        .is_registered(current_shortcut.as_str())
    {
        if let Err(err) = app.global_shortcut().unregister(current_shortcut.as_str()) {
            let _ = app.global_shortcut().unregister(normalized.as_str());
            return Err(format!("failed to unregister old shortcut: {err}"));
        }
    }

    if let Err(err) = persist_hotkeys(&app, &next_hotkeys) {
        let _ = app.global_shortcut().unregister(normalized.as_str());
        let _ = app.global_shortcut().register(current_shortcut.as_str());
        return Err(err);
    }

    let mut hotkeys = state
        .hotkeys
        .lock()
        .map_err(|_| "failed to update hotkey".to_string())?;
    *hotkeys = std::mem::take(&mut next_hotkeys);

    Ok(format_hotkey_for_client(normalized.as_str()))
}

pub fn register_initial<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    load_persisted_hotkeys(app)?;

    let hotkeys = app
        .state::<AppState>()
        .hotkeys
        .lock()
        .map_err(|_| "failed to read startup hotkeys".to_string())?
        .clone();

    for (action, shortcut) in hotkeys {
        if let Err(err) = app.global_shortcut().register(shortcut.as_str()) {
            eprintln!(
                "[hotkeys] startup shortcut '{}' for action '{}' unavailable: {}",
                shortcut, action, err
            );
        }
    }

    Ok(())
}

pub fn on_pressed<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut) {
    let pressed_shortcut = shortcut.into_string();

    let action = match app.state::<AppState>().hotkeys.lock() {
        Ok(hotkeys) => hotkeys.iter().find_map(|(action, configured_shortcut)| {
            if configured_shortcut == &pressed_shortcut {
                Some(action.clone())
            } else {
                None
            }
        }),
        Err(_) => None,
    };

    match action.as_deref() {
        Some(HOTKEY_ACTION_SCREENSHOT) => trigger_screenshot(app),
        Some(unknown_action) => {
            eprintln!(
                "[hotkeys] no runtime handler implemented for action '{}'",
                unknown_action
            );
        }
        None => {}
    }
}

fn load_persisted_hotkeys<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to load settings store: {err}"))?;

    let saved_hotkeys = store
        .get(HOTKEYS_STORE_KEY)
        .and_then(|value| serde_json::from_value::<HashMap<String, String>>(value).ok());

    let resolved_hotkeys = resolve_hotkeys(saved_hotkeys.as_ref());

    let app_state = app.state::<AppState>();
    let mut hotkeys = app_state
        .hotkeys
        .lock()
        .map_err(|_| "failed to apply persisted hotkeys".to_string())?;
    *hotkeys = resolved_hotkeys;

    Ok(())
}

fn resolve_hotkeys(saved_hotkeys: Option<&HashMap<String, String>>) -> HashMap<String, String> {
    let mut resolved = HashMap::new();
    let mut used_shortcuts = HashSet::new();

    for (action, default_shortcut) in HOTKEY_DEFINITIONS {
        let fallback =
            normalize_hotkey(default_shortcut).unwrap_or_else(|_| default_shortcut.to_string());

        let mut selected = saved_hotkeys
            .and_then(|saved| saved.get(action))
            .and_then(|saved_shortcut| normalize_hotkey(saved_shortcut).ok())
            .unwrap_or_else(|| fallback.clone());

        if used_shortcuts.contains(&selected) {
            selected = fallback;
        }

        if used_shortcuts.insert(selected.clone()) {
            resolved.insert(action.to_string(), selected);
        }
    }

    resolved
}

fn persist_hotkeys<R: Runtime>(
    app: &AppHandle<R>,
    hotkeys: &HashMap<String, String>,
) -> Result<(), String> {
    let serialized = serde_json::to_value(hotkeys)
        .map_err(|err| format!("failed to serialize hotkeys: {err}"))?;

    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| format!("failed to open settings store: {err}"))?;
    store.set(HOTKEYS_STORE_KEY, serialized);
    store
        .save()
        .map_err(|err| format!("failed to save hotkeys: {err}"))
}

fn trigger_screenshot<R: Runtime>(app: &AppHandle<R>) {
    let app_state = app.state::<AppState>();
    let toggle_mode_enabled = app_state
        .overlay_toggle_mode
        .lock()
        .map(|value| *value)
        .unwrap_or(false);

    if toggle_mode_enabled {
        if app_state
            .overlay_toggle_in_flight
            .swap(true, Ordering::AcqRel)
        {
            return;
        }

        let app_handle = app.clone();
        std::thread::spawn(move || {
            if let Err(err) = ocr::toggle_overlay_hotkey(&app_handle) {
                eprintln!("toggle overlay failed: {err}");
            }
            app_handle
                .state::<AppState>()
                .overlay_toggle_in_flight
                .store(false, Ordering::Release);
        });
        return;
    }

    let app_handle = app.clone();
    std::thread::spawn(move || {
        if let Err(err) = ocr::capture_active_window(&app_handle) {
            eprintln!("ocr failed: {err}");
        }
    });
}

fn default_shortcut_for_action(action: &str) -> Option<&'static str> {
    HOTKEY_DEFINITIONS
        .iter()
        .find_map(|(candidate, default_shortcut)| {
            if *candidate == action {
                Some(*default_shortcut)
            } else {
                None
            }
        })
}

fn ensure_known_action(action: &str) -> Result<(), String> {
    if HOTKEY_DEFINITIONS
        .iter()
        .any(|(candidate, _)| *candidate == action)
    {
        return Ok(());
    }

    Err(format!("unknown hotkey action: {action}"))
}

fn normalize_action(action: &str) -> String {
    action.trim().to_ascii_lowercase()
}

fn format_hotkey_for_client(input: &str) -> String {
    let Ok(shortcut) = Shortcut::from_str(input) else {
        return input.trim().to_ascii_lowercase();
    };

    format_shortcut_for_client(shortcut)
}

fn format_shortcut_for_client(shortcut: Shortcut) -> String {
    let mut segments = Vec::with_capacity(5);

    if shortcut.mods.contains(Modifiers::CONTROL) {
        segments.push("ctrl".to_string());
    }
    if shortcut.mods.contains(Modifiers::ALT) {
        segments.push("alt".to_string());
    }
    if shortcut.mods.contains(Modifiers::SHIFT) {
        segments.push("shift".to_string());
    }
    if shortcut.mods.contains(Modifiers::SUPER) {
        segments.push("super".to_string());
    }

    segments.push(format_key_code_for_client(shortcut.key));
    segments.join("+")
}

fn format_key_code_for_client(code: Code) -> String {
    let key_name = code.to_string();

    if let Some(letter) = key_name.strip_prefix("Key") {
        if letter.len() == 1 && letter.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return letter.to_ascii_lowercase();
        }
    }

    if let Some(digit) = key_name.strip_prefix("Digit") {
        if digit.len() == 1 && digit.chars().all(|ch| ch.is_ascii_digit()) {
            return digit.to_string();
        }
    }

    match key_name.as_str() {
        "ArrowUp" => "up".to_string(),
        "ArrowDown" => "down".to_string(),
        "ArrowLeft" => "left".to_string(),
        "ArrowRight" => "right".to_string(),
        "Escape" => "esc".to_string(),
        _ => key_name.to_ascii_lowercase(),
    }
}

fn normalize_hotkey(input: &str) -> Result<String, String> {
    let cleaned = input.trim();
    let shortcut = Shortcut::from_str(cleaned).map_err(|err| format!("invalid shortcut: {err}"))?;
    Ok(shortcut.into_string())
}
