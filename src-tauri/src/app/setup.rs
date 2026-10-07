//! Application initialization — runs once during Tauri's `setup` hook.

use log::error;
use tauri::{App, Manager};

use crate::state::AppState;
use crate::store_ext::SettingsExt;
use crate::{data::api, desktop::hotkeys, desktop::layer_shell, desktop::window_watcher, ocr};

#[cfg(target_os = "windows")]
use crate::relics::dbwin as relic_rewards;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use crate::relics::visual_detection as relic_visual_detection;

/// Called by Tauri during startup to configure windows, load data, and spawn
/// background tasks.
pub fn init(app: &mut App, is_wayland: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Config disables developer tools during window creation; restore only an explicit opt-in.
    let developer_console_enabled = app
        .handle()
        .get_setting_bool("developer_console_enabled", false);
    if developer_console_enabled {
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = crate::app::developer_console::apply(&handle, true).await {
                error!("could not enable developer console: {error}");
            }
        });
    }
    crate::desktop::window_size::restore_artus_size(app);
    crate::desktop::tray::init(app)?;
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("overlay window not found")?;

    overlay.hide()?;

    let layer_shell_enabled = layer_shell::configure_overlay_window(&overlay)?;

    // Apply input pass-through eagerly on non-Wayland.
    if !is_wayland {
        let _ = overlay.set_ignore_cursor_events(true);
        let _ = overlay.set_focusable(false);

        #[cfg(target_os = "windows")]
        if let Ok(hwnd) = overlay.hwnd() {
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
                };
                let hwnd_val = windows::Win32::Foundation::HWND(hwnd.0 as *mut core::ffi::c_void);
                let ex_style = GetWindowLongW(hwnd_val, GWL_EXSTYLE);
                SetWindowLongW(hwnd_val, GWL_EXSTYLE, ex_style | WS_EX_NOACTIVATE.0 as i32);
            }
        }
    } else if layer_shell_enabled {
        let _ = overlay.set_focusable(false);
    }

    // Load OCR theme colors from embedded TOML
    match ocr::load_primary_theme_options(&app.handle()) {
        Ok(themes) => {
            let map = themes.into_iter().map(|t| (t.name, t.rgb)).collect();
            if let Ok(mut colors) = app.state::<AppState>().ocr_theme_colors.lock() {
                *colors = map;
            }
        }
        Err(err) => error!("failed to load primary themes: {err}"),
    }

    // Network catalogs can take tens of seconds; keep the setup hook free of HTTP work.
    api::start_refresh_loop(app.handle().clone());

    // Spawn background tasks
    #[cfg(target_os = "windows")]
    relic_rewards::spawn_dbwin_listener(app.handle().clone());
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    relic_visual_detection::spawn_visual_listener(app.handle().clone());
    hotkeys::register_initial(app.handle())?;
    window_watcher::spawn_window_watcher(app.handle().clone());

    Ok(())
}
