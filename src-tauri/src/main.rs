mod dictionary;
mod hotkeys;
mod ocr;
mod prices;
mod state;

use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            GlobalShortcutBuilder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        hotkeys::on_pressed(app, shortcut);
                    }
                })
                .build(),
        )
        .manage(AppState::default())
        .setup(|app| {
            let overlay = app
                .get_webview_window("overlay")
                .ok_or("overlay window not found")?;

            overlay.hide()?;
            let _ = overlay.set_ignore_cursor_events(true);
            let _ = overlay.set_focusable(false);

            hotkeys::register_initial(app.handle())?;
            dictionary::refresh_dictionary_on_start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hotkeys::get_hotkey,
            hotkeys::set_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
