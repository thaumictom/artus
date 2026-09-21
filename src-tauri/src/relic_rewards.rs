//! Real-time Warframe relic reward detection through Windows DBWIN.
//!
//! The listener owns the DBWIN shared objects only while automatic detection
//! is enabled. It never attaches to or reads memory from the Warframe process.

use std::time::{Duration, Instant};

use log::{error, info, warn};
use tauri::{AppHandle, Runtime};

use crate::ocr;
use crate::relic_reward_capture;
use crate::store_ext::SettingsExt;

// ── Log markers ───────────────────────────────────────────────────────────────

const GOT_REWARDS_MARKER: &str = "Got rewards";
const SCREEN_SHUTDOWN_MARKER: &str = "Relic reward screen shut down";
const REWARD_CAPTURE_DELAY: Duration = Duration::from_millis(500);

// ── Public API ────────────────────────────────────────────────────────────────

/// Starts the DBWIN listener supervisor. The thread remains idle until automatic
/// detection is enabled.
pub fn spawn_dbwin_listener<R: Runtime + 'static>(app: AppHandle<R>) {
    if let Err(err) = std::thread::Builder::new()
        .name("warframe-dbwin-listener".into())
        .spawn(move || windows_debug_output::run(app))
    {
        error!("failed to spawn DBWIN listener: {err}");
    }
}

fn process_debug_message<R: Runtime>(app: &AppHandle<R>, message: &str) {
    if !app.get_setting_bool("relic_reward_detection", false) {
        return;
    }

    if message.contains(SCREEN_SHUTDOWN_MARKER) {
        info!("detected relic reward screen shutdown via DBWIN, hiding overlay");
        let _ = ocr::hide_overlay(app);
    } else if message.contains(GOT_REWARDS_MARKER) {
        trigger_relic_capture(app);
    }
}

fn trigger_relic_capture<R: Runtime>(app: &AppHandle<R>) {
    relic_reward_capture::trigger(app, "relic_reward_detection", "DBWIN", REWARD_CAPTURE_DELAY);
}

mod windows_debug_output {
    use super::*;
    use std::ffi::OsStr;
    use std::slice;
    use sysinfo::{Pid, ProcessesToUpdate, System};
    use windows::core::w;
    use windows::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, INVALID_HANDLE_VALUE,
        WAIT_OBJECT_0, WAIT_TIMEOUT,
    };
    use windows::Win32::System::Memory::{
        CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ,
        MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
    };
    use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};

    const DBWIN_BUFFER_SIZE: usize = 4096;
    const SETTING_POLL_MS: u32 = 250;
    const RETRY_DELAY: Duration = Duration::from_secs(5);
    const PROCESS_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
    const WARFRAME_PROCESS_NAMES: [&str; 2] = ["Warframe.x64.exe", "Warframe.exe"];

    struct OwnedHandle(HANDLE);

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    struct DbwinListener {
        _mapping: OwnedHandle,
        buffer_ready: OwnedHandle,
        data_ready: OwnedHandle,
        view: MEMORY_MAPPED_VIEW_ADDRESS,
    }

    impl DbwinListener {
        fn create() -> Result<Self, String> {
            unsafe {
                let mapping = CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    None,
                    PAGE_READWRITE,
                    0,
                    DBWIN_BUFFER_SIZE as u32,
                    w!("DBWIN_BUFFER"),
                )
                .map_err(|err| format!("create DBWIN buffer: {err}"))?;
                let mapping = claim_new_handle(mapping, "DBWIN_BUFFER")?;

                let buffer_ready = CreateEventW(None, false, false, w!("DBWIN_BUFFER_READY"))
                    .map_err(|err| format!("create DBWIN_BUFFER_READY: {err}"))?;
                let buffer_ready = claim_new_handle(buffer_ready, "DBWIN_BUFFER_READY")?;

                let data_ready = CreateEventW(None, false, false, w!("DBWIN_DATA_READY"))
                    .map_err(|err| format!("create DBWIN_DATA_READY: {err}"))?;
                let data_ready = claim_new_handle(data_ready, "DBWIN_DATA_READY")?;

                let view = MapViewOfFile(mapping.0, FILE_MAP_READ, 0, 0, DBWIN_BUFFER_SIZE);
                if view.Value.is_null() {
                    return Err(format!(
                        "map DBWIN buffer: {}",
                        windows::core::Error::from_thread()
                    ));
                }

                SetEvent(buffer_ready.0)
                    .map_err(|err| format!("signal DBWIN_BUFFER_READY: {err}"))?;

                Ok(Self {
                    _mapping: mapping,
                    buffer_ready,
                    data_ready,
                    view,
                })
            }
        }

        fn next_message(&self) -> Result<Option<(u32, String)>, String> {
            unsafe {
                match WaitForSingleObject(self.data_ready.0, SETTING_POLL_MS) {
                    WAIT_OBJECT_0 => {
                        let buffer =
                            slice::from_raw_parts(self.view.Value.cast::<u8>(), DBWIN_BUFFER_SIZE);
                        let process_id = u32::from_ne_bytes(
                            buffer[..std::mem::size_of::<u32>()]
                                .try_into()
                                .expect("DBWIN process id has a fixed size"),
                        );
                        let message_bytes = &buffer[std::mem::size_of::<u32>()..];
                        let end = message_bytes
                            .iter()
                            .position(|byte| *byte == 0)
                            .unwrap_or(message_bytes.len());
                        let message = String::from_utf8_lossy(&message_bytes[..end]).into_owned();

                        // Release Warframe immediately; parsing and OCR happen afterwards.
                        SetEvent(self.buffer_ready.0)
                            .map_err(|err| format!("signal DBWIN_BUFFER_READY: {err}"))?;
                        Ok(Some((process_id, message)))
                    }
                    WAIT_TIMEOUT => Ok(None),
                    status => Err(format!("wait for DBWIN_DATA_READY returned {}", status.0)),
                }
            }
        }
    }

    impl Drop for DbwinListener {
        fn drop(&mut self) {
            unsafe {
                let _ = UnmapViewOfFile(self.view);
            }
        }
    }

    unsafe fn claim_new_handle(handle: HANDLE, name: &str) -> Result<OwnedHandle, String> {
        let already_exists = unsafe { GetLastError() == ERROR_ALREADY_EXISTS };
        let handle = OwnedHandle(handle);
        if already_exists {
            Err(format!("{name} is already owned by another debug listener"))
        } else {
            Ok(handle)
        }
    }

    fn is_warframe_process(system: &System, process_id: u32) -> bool {
        system
            .process(Pid::from_u32(process_id))
            .is_some_and(|process| {
                WARFRAME_PROCESS_NAMES
                    .iter()
                    .any(|name| process.name() == OsStr::new(name))
            })
    }

    pub(super) fn run<R: Runtime + 'static>(app: AppHandle<R>) {
        let mut unavailable_logged = false;

        loop {
            while !app.get_setting_bool("relic_reward_detection", false) {
                std::thread::sleep(Duration::from_millis(SETTING_POLL_MS.into()));
            }

            let listener = match DbwinListener::create() {
                Ok(listener) => {
                    info!("listening for real-time Warframe debug output via DBWIN");
                    unavailable_logged = false;
                    listener
                }
                Err(err) => {
                    if !unavailable_logged {
                        warn!("DBWIN listener unavailable: {err}");
                        unavailable_logged = true;
                    }
                    std::thread::sleep(RETRY_DELAY);
                    continue;
                }
            };

            let mut system = System::new_all();
            let mut last_process_refresh = Instant::now();

            while app.get_setting_bool("relic_reward_detection", false) {
                match listener.next_message() {
                    Ok(Some((process_id, message))) => {
                        if last_process_refresh.elapsed() >= PROCESS_REFRESH_INTERVAL {
                            system.refresh_processes(ProcessesToUpdate::All, true);
                            last_process_refresh = Instant::now();
                        }

                        if is_warframe_process(&system, process_id) {
                            process_debug_message(&app, &message);
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        error!("DBWIN listener failed: {err}");
                        break;
                    }
                }
            }

            // Drop all DBWIN handles promptly when the setting is disabled.
            drop(listener);
            info!("stopped real-time Warframe debug output listener");
        }
    }
}
