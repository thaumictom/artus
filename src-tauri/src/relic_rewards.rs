use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};

use crate::ocr;
use crate::state::AppState;

const GOT_REWARDS_MARKER: &str = "ProjectionRewardChoice.lua: Got rewards";
const SCREEN_SHUTDOWN_MARKER: &str = "ProjectionRewardChoice.lua: Relic reward screen shut down";
const DISABLED_SLEEP: Duration = Duration::from_millis(1000);
const IDLE_SLEEP: Duration = Duration::from_millis(100);
const ERROR_SLEEP: Duration = Duration::from_millis(500);

pub fn spawn_log_tailer<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        let mut last_path = String::new();
        let mut last_pos = 0;
        let mut file: Option<File> = None;

        loop {
            let (enabled, path) = get_tailer_config(&app);

            if !enabled || path.is_empty() {
                reset_tailer_state(&mut file, &mut last_path, &mut last_pos);
                std::thread::sleep(DISABLED_SLEEP);
                continue;
            }

            if path != last_path {
                open_new_log_file(&path, &mut file, &mut last_path, &mut last_pos);
            }

            let data_to_process = read_new_data(&mut file, &last_path, &mut last_pos);

            if !data_to_process.is_empty() {
                process_log_data(&app, &data_to_process);
            }

            std::thread::sleep(IDLE_SLEEP);
        }
    });
}

fn get_tailer_config<R: Runtime>(app: &AppHandle<R>) -> (bool, String) {
    let state = app.state::<AppState>();
    let enabled = state
        .relic_reward_detection
        .lock()
        .map(|value| *value)
        .unwrap_or(false);
    let path = state
        .warframe_log_path
        .lock()
        .map(|value| value.clone())
        .unwrap_or_default();
    (enabled, path)
}

fn reset_tailer_state(file: &mut Option<File>, last_path: &mut String, last_pos: &mut u64) {
    if file.is_some() {
        *file = None;
        last_path.clear();
        *last_pos = 0;
    }
}

fn open_new_log_file(
    path: &str,
    file: &mut Option<File>,
    last_path: &mut String,
    last_pos: &mut u64,
) {
    *last_path = path.to_string();
    *file = File::open(&*last_path).ok();
    *last_pos = file
        .as_mut()
        .and_then(|f| f.seek(SeekFrom::End(0)).ok())
        .unwrap_or(0);

    if file.is_some() {
        println!(
            "[relic_rewards] tailing log: {}, starting at {} bytes",
            last_path, last_pos
        );
    }
}

fn read_new_data(file: &mut Option<File>, last_path: &str, last_pos: &mut u64) -> Vec<u8> {
    let mut data = Vec::new();

    if let Some(f) = file.as_mut() {
        if let Ok(current_len) = f.seek(SeekFrom::End(0)) {
            if current_len > *last_pos {
                if f.seek(SeekFrom::Start(*last_pos)).is_ok() {
                    if f.read_to_end(&mut data).is_ok() {
                        *last_pos = current_len;
                    }
                }
            } else if current_len < *last_pos {
                *last_pos = current_len; // Truncated/rotated
            }
        }
    } else if !last_path.is_empty() {
        std::thread::sleep(ERROR_SLEEP);
        *file = File::open(last_path).ok();
        *last_pos = file
            .as_mut()
            .and_then(|f| f.seek(SeekFrom::End(0)).ok())
            .unwrap_or(0);
    }

    data
}

fn process_log_data<R: Runtime>(app: &AppHandle<R>, data: &[u8]) {
    let content = String::from_utf8_lossy(data);

    if content.contains(SCREEN_SHUTDOWN_MARKER) {
        println!("[relic_rewards] detected screen shut down, hiding overlay");
        if let Some(overlay) = app.get_webview_window("overlay") {
            let _ = overlay.hide();
        }
    } else if content.contains(GOT_REWARDS_MARKER) {
        println!("[relic_rewards] detected rewards, triggering OCR");
        let app_handle = app.clone();
        std::thread::spawn(move || {
            if let Err(err) = ocr::capture_active_window_with_mode(&app_handle, false) {
                eprintln!("[relic_rewards] OCR failed: {err}");
            }
        });
    }
}
