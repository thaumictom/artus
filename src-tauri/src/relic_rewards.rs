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
            let (enabled, path) = {
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
            };

            if !enabled || path.is_empty() {
                if file.is_some() {
                    file = None;
                    last_path.clear();
                    last_pos = 0;
                }
                std::thread::sleep(DISABLED_SLEEP);
                continue;
            }

            if path != last_path {
                last_path = path.clone();
                file = File::open(&last_path).ok();
                last_pos = file
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

            let mut data_to_process = Vec::new();
            if let Some(f) = file.as_mut() {
                if let Ok(current_len) = f.seek(SeekFrom::End(0)) {
                    if current_len > last_pos {
                        if f.seek(SeekFrom::Start(last_pos)).is_ok() {
                            let mut buf = Vec::new();
                            if f.read_to_end(&mut buf).is_ok() {
                                data_to_process = buf;
                                last_pos = current_len;
                            }
                        }
                    } else if current_len < last_pos {
                        last_pos = current_len; // Truncated/rotated
                    }
                }
            } else if !last_path.is_empty() {
                std::thread::sleep(ERROR_SLEEP);
                file = File::open(&last_path).ok();
                last_pos = file
                    .as_mut()
                    .and_then(|f| f.seek(SeekFrom::End(0)).ok())
                    .unwrap_or(0);
            }

            if !data_to_process.is_empty() {
                let content = String::from_utf8_lossy(&data_to_process);

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

            std::thread::sleep(IDLE_SLEEP);
        }
    });
}
