//! Central application error type.
//!
//! Frontend code consumes these via Tauri commands, which serialize `Error`
//! into a JSON-friendly shape. Keep variants stable — they double as an
//! API contract with the React layer.

use serde::Serialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("storage error: {0}")]
    Storage(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("card not found: {0}")]
    CardNotFound(String),

    #[error("unsupported game: {0}")]
    UnsupportedGame(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("image decode failed: {0}")]
    ImageDecode(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// JSON representation sent to the frontend.
///
/// A flat { kind, message } object is easy to pattern-match from TS.
impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let kind = match self {
            Error::Storage(_) => "storage",
            Error::Network(_) => "network",
            Error::CardNotFound(_) => "card_not_found",
            Error::UnsupportedGame(_) => "unsupported_game",
            Error::InvalidInput(_) => "invalid_input",
            Error::ImageDecode(_) => "image_decode",
            Error::Internal(_) => "internal",
        };
        let mut st = s.serialize_struct("Error", 2)?;
        st.serialize_field("kind", kind)?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

// Convenience From impls so `?` works from the common dependencies.

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Storage(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Network(e.to_string())
    }
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::ImageDecode(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Internal(format!("json: {e}"))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Internal(format!("io: {e}"))
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Internal(e.to_string())
    }
}
