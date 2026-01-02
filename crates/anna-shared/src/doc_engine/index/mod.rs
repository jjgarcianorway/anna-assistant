//! Documentation index for storage and retrieval (v0.0.429).
//!
//! Lightweight on-disk index for full-text search.

mod operations;
mod search;
mod storage;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types and functions
pub use storage::{get_storage_path, get_wiki_cache_path};
pub use types::{DocIndex, IndexError};
