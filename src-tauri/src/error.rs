//! Centralized error handling for the Artus backend.
//!
//! All Tauri commands and internal functions return [`AppResult<T>`] instead of
//! `Result<T, String>`. The error is serialized as a plain string over IPC, so
//! the frontend sees no change.

use serde::Serialize;
use thiserror::Error;

/// Application-wide error type.
#[derive(Debug, Error)]
pub enum AppError {
    /// Catch-all for human-readable error messages.
    #[error("{0}")]
    Message(String),

    /// A `std::sync::Mutex` was poisoned.
    #[error("internal lock poisoned")]
    LockPoisoned,

    /// A Tauri webview window with the given label was not found.
    #[error("window '{0}' not found")]
    WindowNotFound(String),

    /// An HTTP request failed.
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON (de)serialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML deserialization failed.
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),

    /// An I/O operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    /// Convenience constructor from any displayable value.
    pub fn msg(value: impl std::fmt::Display) -> Self {
        Self::Message(value.to_string())
    }
}

/// Convert a poisoned-lock error into [`AppError::LockPoisoned`].
impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::LockPoisoned
    }
}

/// Tauri commands require the error type to be serializable.
/// We serialize every variant as its Display string.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Shorthand used throughout the crate.
pub type AppResult<T> = Result<T, AppError>;
