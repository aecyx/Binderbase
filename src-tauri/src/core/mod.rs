//! Core domain model — game-agnostic types used everywhere.
//!
//! Keep this module free of I/O, HTTP, SQLite, or platform-specific code.
//! Those belong in `storage`, `games`, `scanning`, `collection`, and `pricing`.

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::{Card, CardCondition, CardId, Game, PrintingId};
