//! Wayland layer-shell integration for the overlay window.
//!
//! On Wayland with the `wayland-layer-shell` feature enabled, the overlay is
//! configured as a layer-shell surface so it floats above fullscreen games.
//! On other platforms, every function is a no-op returning `Ok(false)`.

#[cfg(target_os = "linux")]
use std::env;

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
use cairo::{RectangleInt, Region};

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
use gtk_layer_shell::{Edge, Layer, LayerShell};

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
use gtk::prelude::WidgetExt;

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
use std::sync::Arc;

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
use tauri::{Runtime, WebviewWindow};

// ── Session detection ─────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub fn is_wayland_session() -> bool {
    env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var("XDG_SESSION_TYPE")
            .map(|v| v.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
pub fn is_wayland_session() -> bool {
    false
}

// ── GTK main-thread dispatch helper ───────────────────────────────────────────

/// Runs a closure on the GTK main thread via `WebviewWindow::run_on_main_thread`.
/// Returns `true` if the closure executed and the `applied` flag was set.
///
/// This extracts the repeated `Arc<AtomicBool>` + `run_on_main_thread` pattern.
#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
fn dispatch_on_main_thread<R: Runtime>(
    overlay: &WebviewWindow<R>,
    action: impl FnOnce(WebviewWindow<R>) -> bool + Send + 'static,
) -> Result<bool, String> {
    let handle = overlay.clone();
    let applied = Arc::new(AtomicBool::new(false));
    let flag = applied.clone();

    overlay
        .run_on_main_thread(move || {
            if action(handle) {
                flag.store(true, Ordering::Relaxed);
            }
        })
        .map_err(|err| format!("GTK main-thread dispatch failed: {err}"))?;

    Ok(applied.load(Ordering::Relaxed))
}

// ── Overlay configuration ─────────────────────────────────────────────────────

/// Configures the overlay as a Wayland layer-shell surface.
#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
pub fn configure_overlay_window<R: Runtime>(overlay: &WebviewWindow<R>) -> Result<bool, String> {
    if !is_wayland_session() {
        return Ok(false);
    }

    dispatch_on_main_thread(overlay, |handle| {
        if let Ok(gtk_window) = handle.gtk_window() {
            gtk_window.init_layer_shell();
            gtk_window.set_namespace("artus-overlay");
            gtk_window.set_layer(Layer::Overlay);
            gtk_window.set_exclusive_zone(0);
            gtk_window.set_keyboard_interactivity(false);

            // Anchor to top-left; position is controlled via margins
            gtk_window.set_anchor(Edge::Left, true);
            gtk_window.set_anchor(Edge::Top, true);
            gtk_window.set_anchor(Edge::Right, false);
            gtk_window.set_anchor(Edge::Bottom, false);
            true
        } else {
            false
        }
    })
}

#[cfg(not(all(target_os = "linux", feature = "wayland-layer-shell")))]
pub fn configure_overlay_window<R>(_overlay: &R) -> Result<bool, String> {
    Ok(false)
}

// ── Overlay geometry ──────────────────────────────────────────────────────────

/// Sets the overlay position using layer-shell margins.
#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
pub fn set_overlay_geometry<R: Runtime>(
    overlay: &WebviewWindow<R>,
    x: i32,
    y: i32,
) -> Result<bool, String> {
    if !is_wayland_session() {
        return Ok(false);
    }

    let left_margin = x.max(0);
    let top_margin = y.max(0);

    dispatch_on_main_thread(overlay, move |handle| {
        if let Ok(gtk_window) = handle.gtk_window() {
            gtk_window.set_layer_shell_margin(Edge::Left, left_margin);
            gtk_window.set_layer_shell_margin(Edge::Top, top_margin);
            true
        } else {
            false
        }
    })
}

#[cfg(not(all(target_os = "linux", feature = "wayland-layer-shell")))]
pub fn set_overlay_geometry<R>(_overlay: &R, _x: i32, _y: i32) -> Result<bool, String> {
    Ok(false)
}

// ── Click-through ─────────────────────────────────────────────────────────────

/// Forces click-through on the overlay using GDK input regions.
#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
pub fn force_click_through<R: Runtime>(overlay: &WebviewWindow<R>) -> Result<bool, String> {
    if !is_wayland_session() {
        return Ok(false);
    }

    dispatch_on_main_thread(overlay, |handle| {
        if let Ok(gtk_window) = handle.gtk_window() {
            if let Some(gdk_window) = gtk_window.window() {
                gdk_window.set_pass_through(true);

                // Tiny input region as fallback for compositors that don't fully
                // honor pass-through on layer surfaces.
                let tiny = Region::create_rectangle(&RectangleInt::new(0, 0, 1, 1));
                gdk_window.input_shape_combine_region(&tiny, 0, 0);
                return true;
            }
        }
        false
    })
}

#[cfg(not(all(target_os = "linux", feature = "wayland-layer-shell")))]
pub fn force_click_through<R>(_overlay: &R) -> Result<bool, String> {
    Ok(false)
}
