use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};

use crate::ocr;
use crate::state::AppState;

const GOT_REWARDS_MARKER: &str = "ProjectionRewardChoice.lua: Got rewards";
const SCREEN_SHUTDOWN_MARKER: &str = "ProjectionRewardChoice.lua: Relic reward screen shut down";
const READER_CAPACITY: usize = 64 * 1024;
const DISABLED_SLEEP: Duration = Duration::from_millis(500);
const ERROR_SLEEP: Duration = Duration::from_millis(250);
const IDLE_SLEEP: Duration = Duration::from_millis(75);

pub fn spawn_log_tailer<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        let mut last_path = String::new();
        let mut reader: Option<BufReader<File>> = None;
        let mut line = String::new();

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
                last_path.clear();
                reader = None;
                std::thread::sleep(DISABLED_SLEEP);
                continue;
            }

            if path != last_path {
                last_path = path.clone();
                reader = open_reader(&last_path, true);
            }

            let Some(active_reader) = reader.as_mut() else {
                std::thread::sleep(ERROR_SLEEP);
                continue;
            };

            line.clear();
            match active_reader.read_line(&mut line) {
                Ok(0) => {
                    std::thread::sleep(IDLE_SLEEP);
                }
                Ok(_) => {
                    if line.contains(GOT_REWARDS_MARKER) {
                        println!("[relic_rewards] detected rewards, triggering OCR");
                        let app_handle = app.clone();
                        std::thread::spawn(move || {
                            if let Err(err) =
                                ocr::capture_active_window_with_mode(&app_handle, false)
                            {
                                eprintln!("[relic_rewards] OCR failed: {err}");
                            }
                        });
                    } else if line.contains(SCREEN_SHUTDOWN_MARKER) {
                        println!("[relic_rewards] detected screen shut down, hiding overlay");
                        if let Some(overlay) = app.get_webview_window("overlay") {
                            let _ = overlay.hide();
                        }
                    }
                }
                Err(_) => {
                    reader = open_reader(&last_path, true);
                    std::thread::sleep(ERROR_SLEEP);
                }
            }
        }
    });
}

fn open_reader(path: &str, start_at_end: bool) -> Option<BufReader<File>> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::with_capacity(READER_CAPACITY, file);

    if start_at_end {
        let len = reader
            .get_ref()
            .metadata()
            .map(|meta| meta.len())
            .unwrap_or(0);
        let _ = reader.seek(SeekFrom::Start(len));
    }

    Some(reader)
}
