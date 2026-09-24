//! Main-window tray behavior. The overlay is never shown or focused here.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Window, WindowEvent,
};

use crate::store_ext::SettingsExt;

const HIDE_TO_TRAY_KEY: &str = "hide_to_tray_on_close";

pub fn init(app: &App) -> tauri::Result<()> {
    let menu = Menu::new(app)?;
    #[cfg(target_os = "linux")]
    menu.append(&MenuItem::with_id(app, "open", "Open Artus", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "restart", "Restart Artus", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit Artus", true, None::<&str>)?)?;

    let icon = app.default_window_icon().cloned().ok_or(tauri::Error::FailedToReceiveMessage)?;
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Artus")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            #[cfg(target_os = "linux")]
            "open" => show_artus(app),
            "restart" => app.restart(),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_artus(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn show_artus(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("artus") {
        if let Err(error) = window.show().and_then(|_| window.set_focus()) {
            log::warn!("could not show Artus from tray: {error}");
        }
    }
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != "artus" {
        return;
    }
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.app_handle().get_setting_bool(HIDE_TO_TRAY_KEY, true) {
            match window.hide() {
                Ok(()) => api.prevent_close(),
                Err(error) => {
                    log::warn!("could not hide Artus to tray: {error}");
                    window.app_handle().exit(0);
                }
            }
        } else {
            window.app_handle().exit(0);
        }
    }
}
