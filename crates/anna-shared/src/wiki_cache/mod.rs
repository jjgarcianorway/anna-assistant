//! Arch Wiki Local Cache (v0.0.472).
//!
//! Local caching of Arch Wiki pages for offline knowledge.
//! Per VISION.md: "Store local copies of wiki pages if linked from Arch Wiki"
//!
//! Features:
//! - Download and cache essential wiki pages
//! - Track cache freshness and staleness
//! - Manage cache size and cleanup

mod entry;
mod index;
mod io;
mod stats;
mod utils;

// Re-export public types and functions to maintain the API
pub use entry::WikiCacheEntry;
pub use index::{essential_pages, missing_essential, WikiCacheIndex};
pub use io::{delete_cached, get_cache_path, read_cached, write_cached};
pub use stats::{get_cache_stats, CacheStats};
