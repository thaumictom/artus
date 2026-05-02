//! Global hotkey registration, persistence, and dispatch.
//!
//! Hotkeys are stored in `settings.json` under the `"hotkeys"` key and
//! registered/unregistered with the OS when Warframe gains/loses focus.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::atomic::Ordering;

use log::error;
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_store::StoreExt;

use crate::error::{AppError, AppResult};
use crate::ocr;
use crate::state::AppState;
use crate::store_ext::{SettingsExt, SETTINGS_STORE_PATH};

// ── Hotkey definitions ────────────────────────────────────────────────────────

pub const HOTKEY_ACTION_SCREENSHOT: &str = "screenshot";
const DEFAULT_SCREENSHOT_HOTKEY: &str = "Ctrl+Home";

pub const HOTKEY_ACTION_SCREENSHOT_ADD_TO_INVENTORY: &str = "screenshot_add_inventory";
const DEFAULT_SCREENSHOT_ADD_TO_INVENTORY_HOTKEY: &str = "Ctrl+Shift+Home";

/// All known actions and their default shortcuts.
const HOTKEY_DEFINITIONS: [(&str, &str); 2] = [
    (HOTKEY_ACTION_SCREENSHOT, DEFAULT_SCREENSHOT_HOTKEY),
    (
        HOTKEY_ACTION_SCREENSHOT_ADD_TO_INVENTORY,
        DEFAULT_SCREENSHOT_ADD_TO_INVENTORY_HOTKEY,
    ),
];

const HOTKEYS_STORE_KEY: &str = "hotkeys";

/// Modifier flags in the order they appear in formatted output.
const MODIFIER_NAMES: [(Modifiers, &str); 4] = [
    (Modifiers::CONTROL, "ctrl"),
    (Modifiers::ALT, "alt"),
    (Modifiers::SHIFT, "shift"),
    (Modifiers::SUPER, "super"),
];

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Returns the current shortcut string for a given action (client-friendly format).
#[tauri::command]
pub fn get_hotkey(state: State<'_, AppState>, action: String) -> AppResult<String> {
    let action_key = normalize_action(&action);
    ensure_known_action(&action_key)?;

    let hotkeys = state.hotkeys.lock()?;
    let stored = hotkeys
        .get(action_key.as_str())
        .cloned()
        .or_else(|| default_shortcut_for(&action_key).map(str::to_string))
        .ok_or_else(|| AppError::msg(format!("missing hotkey for action: {action_key}")))?;

    Ok(format_hotkey_for_client(&stored))
}

/// Updates the shortcut for a given action, persists it, and re-registers with the OS.
#[tauri::command]
pub fn set_hotkey<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    action: String,
    hotkey: String,
) -> AppResult<String> {
    let action_key = normalize_action(&action);
    ensure_known_action(&action_key)?;

    let normalized = normalize_hotkey(&hotkey)?;

    // Read current state and prepare the update
    let (current_shortcut, mut next_hotkeys) = {
        let hotkeys = state.hotkeys.lock()?;

        let current = hotkeys
            .get(action_key.as_str())
            .cloned()
            .or_else(|| default_shortcut_for(&action_key).map(str::to_string))
            .ok_or_else(|| AppError::msg(format!("missing hotkey for action: {action_key}")))?;

        // No-op if unchanged
        if current == normalized {
            return Ok(format_hotkey_for_client(&normalized));
        }

        // Reject duplicates
        if hotkeys
            .iter()
            .any(|(a, s)| a != &action_key && s == &normalized)
        {
            return Err(AppError::msg(format!(
                "hotkey '{normalized}' is already used by another action"
            )));
        }

        let mut updated = hotkeys.clone();
        updated.insert(action_key.clone(), normalized.clone());
        (current, updated)
    };

    let is_focused = state.warframe_focused.load(Ordering::Acquire);

    // Register the new shortcut and unregister the old one (only while focused)
    if is_focused {
        app.global_shortcut()
            .register(normalized.as_str())
            .map_err(|err| AppError::msg(format!("failed to register shortcut: {err}")))?;

        if app
            .global_shortcut()
            .is_registered(current_shortcut.as_str())
        {
            if let Err(err) = app.global_shortcut().unregister(current_shortcut.as_str()) {
                // Rollback: undo the new registration
                let _ = app.global_shortcut().unregister(normalized.as_str());
                return Err(AppError::msg(format!(
                    "failed to unregister old shortcut: {err}"
                )));
            }
        }
    }

    // Persist to the store; rollback OS registration on failure
    if let Err(err) = persist_hotkeys(&app, &next_hotkeys) {
        if is_focused {
            let _ = app.global_shortcut().unregister(normalized.as_str());
            let _ = app.global_shortcut().register(current_shortcut.as_str());
        }
        return Err(err);
    }

    // Commit to in-memory state
    *state.hotkeys.lock()? = std::mem::take(&mut next_hotkeys);

    Ok(format_hotkey_for_client(&normalized))
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

/// Loads persisted hotkeys from the store and conditionally registers them
/// (if Warframe is already focused at startup).
pub fn register_initial<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    load_persisted_hotkeys(app)?;

    if app
        .state::<AppState>()
        .warframe_focused
        .load(Ordering::Acquire)
    {
        register_all(app);
    }

    Ok(())
}

/// Registers all hotkeys with the OS. Called when Warframe gains focus.
pub fn register_all<R: Runtime>(app: &AppHandle<R>) {
    with_hotkey_entries(app, |action, shortcut| {
        if let Err(err) = app.global_shortcut().register(shortcut) {
            error!("register '{shortcut}' for '{action}' failed: {err}");
        }
    });
}

/// Unregisters all hotkeys from the OS. Called when Warframe loses focus.
pub fn unregister_all<R: Runtime>(app: &AppHandle<R>) {
    with_hotkey_entries(app, |action, shortcut| {
        if let Err(err) = app.global_shortcut().unregister(shortcut) {
            error!("unregister '{shortcut}' for '{action}' failed: {err}");
        }
    });
}

/// Dispatches a pressed shortcut to the appropriate handler.
pub fn on_pressed<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut) {
    let pressed = shortcut.into_string();

    let action = app.state::<AppState>().hotkeys.lock().ok().and_then(|hk| {
        hk.iter()
            .find(|(_, s)| *s == &pressed)
            .map(|(a, _)| a.clone())
    });

    match action.as_deref() {
        Some(HOTKEY_ACTION_SCREENSHOT) => trigger_screenshot(app),
        Some(HOTKEY_ACTION_SCREENSHOT_ADD_TO_INVENTORY) => {
            spawn_ocr_task(app, ocr::capture_active_window);
        }
        Some(unknown) => error!("no handler for action '{unknown}'"),
        None => {}
    }
}

// ── Hotkey action triggers ────────────────────────────────────────────────────

/// Triggers an OCR screenshot, respecting the overlay toggle mode setting.
fn trigger_screenshot<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let toggle_mode = app.get_setting_bool("overlay_toggle_mode", false);

    if toggle_mode {
        // Debounce: only one toggle operation at a time
        if state.overlay_toggle_in_flight.swap(true, Ordering::AcqRel) {
            return;
        }

        let handle = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            if let Err(err) = ocr::toggle_overlay_hotkey(&handle) {
                error!("toggle overlay failed: {err}");
            }
            handle
                .state::<AppState>()
                .overlay_toggle_in_flight
                .store(false, Ordering::Release);
        });
        return;
    }

    spawn_ocr_task(app, ocr::capture_active_window);
}

/// Spawns a blocking OCR task on the tokio threadpool.
fn spawn_ocr_task<R: Runtime>(app: &AppHandle<R>, task: fn(&AppHandle<R>) -> AppResult<()>)
where
    AppHandle<R>: Send + 'static,
{
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(err) = task(&handle) {
            error!("OCR task failed: {err}");
        }
    });
}

// ── Persistence ───────────────────────────────────────────────────────────────

/// Loads hotkeys from the settings store, falling back to defaults.
fn load_persisted_hotkeys<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    let saved = app.get_setting_json_map(HOTKEYS_STORE_KEY);
    let resolved = resolve_hotkeys(saved.as_ref());
    *app.state::<AppState>().hotkeys.lock()? = resolved;
    Ok(())
}

/// Merges saved hotkeys with defaults, handling duplicates and invalid values.
fn resolve_hotkeys(saved: Option<&HashMap<String, String>>) -> HashMap<String, String> {
    let mut resolved = HashMap::new();
    let mut used = HashSet::new();

    for (action, default_shortcut) in HOTKEY_DEFINITIONS {
        let fallback =
            normalize_hotkey(default_shortcut).unwrap_or_else(|_| default_shortcut.to_string());

        let mut selected = saved
            .and_then(|s| s.get(action))
            .and_then(|v| normalize_hotkey(v).ok())
            .unwrap_or_else(|| fallback.clone());

        // Avoid conflicts: fall back to default if already in use
        if used.contains(&selected) {
            selected = fallback;
        }

        if used.insert(selected.clone()) {
            resolved.insert(action.to_string(), selected);
        }
    }

    resolved
}

/// Writes the hotkey map to the settings store.
fn persist_hotkeys<R: Runtime>(
    app: &AppHandle<R>,
    hotkeys: &HashMap<String, String>,
) -> AppResult<()> {
    let serialized = serde_json::to_value(hotkeys)?;
    let store = app
        .store(SETTINGS_STORE_PATH)
        .map_err(|err| AppError::msg(format!("failed to open settings store: {err}")))?;
    store.set(HOTKEYS_STORE_KEY, serialized);
    store
        .save()
        .map_err(|err| AppError::msg(format!("failed to save hotkeys: {err}")))?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Iterates over all configured hotkeys while holding the lock briefly.
fn with_hotkey_entries<R: Runtime>(app: &AppHandle<R>, mut f: impl FnMut(&str, &str)) {
    let state = app.state::<AppState>();
    let hotkeys = match state.hotkeys.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => return,
    };
    for (action, shortcut) in &hotkeys {
        f(action, shortcut);
    }
}

/// Returns the default shortcut for a known action.
fn default_shortcut_for(action: &str) -> Option<&'static str> {
    HOTKEY_DEFINITIONS
        .iter()
        .find(|(a, _)| *a == action)
        .map(|(_, s)| *s)
}

/// Returns `Err` if the action is not in [`HOTKEY_DEFINITIONS`].
fn ensure_known_action(action: &str) -> AppResult<()> {
    if HOTKEY_DEFINITIONS.iter().any(|(a, _)| *a == action) {
        Ok(())
    } else {
        Err(AppError::msg(format!("unknown hotkey action: {action}")))
    }
}

fn normalize_action(action: &str) -> String {
    action.trim().to_ascii_lowercase()
}

/// Formats a shortcut string for display on the frontend (lowercase, `+`-separated).
fn format_hotkey_for_client(input: &str) -> String {
    match Shortcut::from_str(input) {
        Ok(shortcut) => format_shortcut(shortcut),
        Err(_) => input.trim().to_ascii_lowercase(),
    }
}

fn format_shortcut(shortcut: Shortcut) -> String {
    let mut parts: Vec<&str> = MODIFIER_NAMES
        .iter()
        .filter(|(m, _)| shortcut.mods.contains(*m))
        .map(|(_, name)| *name)
        .collect();

    parts.push(&format_key_code(shortcut.key));
    // Can't push a temporary — collect modifiers then join
    let mods: Vec<String> = MODIFIER_NAMES
        .iter()
        .filter(|(m, _)| shortcut.mods.contains(*m))
        .map(|(_, name)| name.to_string())
        .collect();

    let key = format_key_code(shortcut.key);
    let mut segments = mods;
    segments.push(key);
    segments.join("+")
}

/// Converts a keyboard Code to a client-friendly string.
fn format_key_code(code: Code) -> String {
    let name = code.to_string();

    if let Some(letter) = name.strip_prefix("Key") {
        if letter.len() == 1 && letter.chars().all(|c| c.is_ascii_alphabetic()) {
            return letter.to_ascii_lowercase();
        }
    }

    if let Some(digit) = name.strip_prefix("Digit") {
        if digit.len() == 1 && digit.chars().all(|c| c.is_ascii_digit()) {
            return digit.to_string();
        }
    }

    match name.as_str() {
        "ArrowUp" => "up",
        "ArrowDown" => "down",
        "ArrowLeft" => "left",
        "ArrowRight" => "right",
        "Escape" => "esc",
        _ => return name.to_ascii_lowercase(),
    }
    .to_string()
}

/// Parses and normalizes a shortcut string via the Tauri shortcut parser.
fn normalize_hotkey(input: &str) -> AppResult<String> {
    let shortcut = Shortcut::from_str(input.trim())
        .map_err(|err| AppError::msg(format!("invalid shortcut: {err}")))?;
    Ok(shortcut.into_string())
}
