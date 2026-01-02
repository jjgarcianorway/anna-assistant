//! Cached documentation snippets.

use serde::{Deserialize, Serialize};

/// Cached documentation snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDoc {
    pub topic: String,
    pub source: String,
    pub snippet: String,
    pub cached_at: u64,
}
