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

#[cfg(target_os = "linux")]
pub fn is_wayland_session() -> bool {
    env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var("XDG_SESSION_TYPE")
            .map(|value| value.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
pub fn is_wayland_session() -> bool {
    false
}

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
pub fn configure_overlay_window<R: Runtime>(overlay: &WebviewWindow<R>) -> Result<bool, String> {
    if !is_wayland_session() {
        return Ok(false);
    }

    let overlay_handle = overlay.clone();
    let applied = Arc::new(AtomicBool::new(false));
    let applied_flag = applied.clone();

    overlay
        .run_on_main_thread(move || {
            if let Ok(gtk_window) = overlay_handle.gtk_window() {
                gtk_window.init_layer_shell();
                gtk_window.set_namespace("artus-overlay");
                gtk_window.set_layer(Layer::Overlay);
                gtk_window.set_exclusive_zone(0);
                gtk_window.set_keyboard_interactivity(false);

                // Anchor to the top-left corner so we can drive offset using margins.
                gtk_window.set_anchor(Edge::Left, true);
                gtk_window.set_anchor(Edge::Top, true);
                gtk_window.set_anchor(Edge::Right, false);
                gtk_window.set_anchor(Edge::Bottom, false);

                applied_flag.store(true, Ordering::Relaxed);
            }
        })
        .map_err(|err| format!("failed to apply layer-shell to overlay: {err}"))?;

    Ok(applied.load(Ordering::Relaxed))
}

#[cfg(not(all(target_os = "linux", feature = "wayland-layer-shell")))]
pub fn configure_overlay_window<R>(_overlay: &R) -> Result<bool, String> {
    Ok(false)
}

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
    let overlay_handle = overlay.clone();
    let applied = Arc::new(AtomicBool::new(false));
    let applied_flag = applied.clone();

    overlay
        .run_on_main_thread(move || {
            if let Ok(gtk_window) = overlay_handle.gtk_window() {
                gtk_window.set_layer_shell_margin(Edge::Left, left_margin);
                gtk_window.set_layer_shell_margin(Edge::Top, top_margin);

                applied_flag.store(true, Ordering::Relaxed);
            }
        })
        .map_err(|err| format!("failed to set layer-shell overlay margins: {err}"))?;

    Ok(applied.load(Ordering::Relaxed))
}

#[cfg(not(all(target_os = "linux", feature = "wayland-layer-shell")))]
pub fn set_overlay_geometry<R>(_overlay: &R, _x: i32, _y: i32) -> Result<bool, String> {
    Ok(false)
}

#[cfg(all(target_os = "linux", feature = "wayland-layer-shell"))]
pub fn force_click_through<R: Runtime>(overlay: &WebviewWindow<R>) -> Result<bool, String> {
    if !is_wayland_session() {
        return Ok(false);
    }

    let overlay_handle = overlay.clone();
    let applied = Arc::new(AtomicBool::new(false));
    let applied_flag = applied.clone();

    overlay
        .run_on_main_thread(move || {
            if let Ok(gtk_window) = overlay_handle.gtk_window() {
                if let Some(gdk_window) = gtk_window.window() {
                    gdk_window.set_pass_through(true);

                    // Keep only a tiny input region as a fallback for compositors
                    // that do not fully honor pass-through on layer surfaces.
                    let tiny_region = Region::create_rectangle(&RectangleInt::new(0, 0, 1, 1));
                    gdk_window.input_shape_combine_region(&tiny_region, 0, 0);

                    applied_flag.store(true, Ordering::Relaxed);
                }
            }
        })
        .map_err(|err| format!("failed to force click-through: {err}"))?;

    Ok(applied.load(Ordering::Relaxed))
}

#[cfg(not(all(target_os = "linux", feature = "wayland-layer-shell")))]
pub fn force_click_through<R>(_overlay: &R) -> Result<bool, String> {
    Ok(false)
}
