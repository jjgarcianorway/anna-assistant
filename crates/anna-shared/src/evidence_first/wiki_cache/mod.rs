//! ArchWiki Cache - Offline Wiki Search (v0.0.435).
//!
//! Local cache of Arch Wiki pages for offline evidence retrieval.
//! Updated via `annactl wiki update`.

mod error;
mod storage;
mod types;

pub use error::CacheError;
pub use storage::WikiCache;
pub use types::{
    CacheStats, WikiPage, WikiSearchHit, WikiSearchResult, WikiSection, ESSENTIAL_PAGES,
};

/// Wiki cache directory.
pub const WIKI_CACHE_DIR: &str = "/var/lib/anna/wiki";
