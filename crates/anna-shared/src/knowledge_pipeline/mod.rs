//! Knowledge Pipeline (v0.0.432).
//!
//! Unified knowledge fetching with strict priority ordering:
//! 1. Probes (live system data) - highest trust
//! 2. Local docs (man, --help, /usr/share/doc)
//! 3. Cached wiki (Arch Wiki offline snapshots)
//! 4. Remote (disabled by default)
//!
//! LLMs interpret and synthesize; they don't memorize facts.
//! Successful research becomes reusable parametric recipes.

mod clarification;
mod fetcher;
mod learning;
mod research;
mod sources;
mod tests;
mod wiki_sync;

pub use clarification::{ClarificationProtocol, ClarificationRequest, ClarificationResponse};
pub use fetcher::{FetchConfig, FetchResult, KnowledgeFetcher};
pub use learning::{LearningLoop, LearningOutcome, RecipeStats};
pub use research::{ResearchOutcome, ResearchPattern, ResearchRequest};
pub use sources::{Citation, KnowledgeSource, SourcePriority, SourceResult};
pub use wiki_sync::{SyncStatus, WikiSyncConfig, WikiSyncer};

/// Maximum age for cached wiki content (7 days).
pub const WIKI_CACHE_MAX_AGE_SECS: u64 = 7 * 24 * 60 * 60;

/// Default path for wiki cache.
pub const WIKI_CACHE_DIR: &str = "wiki_cache";

/// Minimum confidence to use a source without verification.
pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.8;

/// Maximum sources to consult before giving up.
pub const MAX_SOURCE_LOOKUPS: usize = 5;
