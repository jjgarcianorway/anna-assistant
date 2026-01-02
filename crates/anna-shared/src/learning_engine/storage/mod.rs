//! Recipe storage for learning engine (v0.0.427).
//!
//! Persistent storage for learned recipes with:
//! - JSON file storage
//! - Domain-based indexing
//! - Intent-based lookup
//! - Version tracking

mod io;
mod library;

#[cfg(test)]
mod tests;

pub use io::*;
pub use library::*;

/// Get current Unix epoch seconds
pub(crate) fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
