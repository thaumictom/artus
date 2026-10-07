//! Relic reward detection, capture coordination, and inventory additions.
#[cfg(target_os = "windows")]
pub mod auto_add;
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub mod capture;
#[cfg(target_os = "windows")]
pub mod dbwin;
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub mod visual_detection;
