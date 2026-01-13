//! Cache type definitions and global statics.

use anna_shared::config::{PerformanceConfig, WikiConfig};
use anna_shared::recipe::RecipeBook;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::RwLock;
use std::time::Instant;

/// Cached performance config (loaded once at startup)
pub static PERF_CONFIG: RwLock<Option<PerformanceConfig>> = RwLock::new(None);

/// Cached wiki config (embeddings setting)
pub static WIKI_CONFIG: RwLock<Option<WikiConfig>> = RwLock::new(None);

/// Command output cache
pub static COMMAND_CACHE: RwLock<Option<HashMap<String, CachedOutput>>> = RwLock::new(None);

/// Answer cache (for repeated questions)
pub static ANSWER_CACHE: RwLock<Option<HashMap<String, CachedAnswer>>> = RwLock::new(None);

/// LLM response memoization cache
pub static LLM_MEMO_CACHE: RwLock<Option<HashMap<u64, CachedLlmResponse>>> = RwLock::new(None);

/// Global command failure tracking
pub static COMMAND_FAILURE_CACHE: RwLock<Option<HashMap<String, CommandFailureRecord>>> =
    RwLock::new(None);

/// Cached recipe book
pub static RECIPE_BOOK_CACHE: RwLock<Option<CachedRecipeBook>> = RwLock::new(None);

/// Wiki search circuit breaker state
pub static WIKI_FAILURES: AtomicU32 = AtomicU32::new(0);
pub static WIKI_CIRCUIT_OPENED_AT: AtomicU64 = AtomicU64::new(0);

/// Intent classification cache
pub static INTENT_CACHE: RwLock<Option<HashMap<String, CachedIntent>>> = RwLock::new(None);

/// Session-level command failure cache
pub static FAILURE_CACHE: RwLock<Option<HashMap<String, CommandFailure>>> = RwLock::new(None);

/// Wiki search result cache
pub static WIKI_CACHE: RwLock<Option<HashMap<String, CachedWikiResult>>> = RwLock::new(None);

/// In-flight request deduplication
pub static INFLIGHT_REQUESTS: RwLock<Option<HashMap<String, InflightRequest>>> = RwLock::new(None);

// Constants
pub const RECIPE_BOOK_TTL_SECS: u64 = 600;
pub const MAX_ANSWER_CACHE_SIZE: usize = 200;
pub const FAILURE_CACHE_TTL_SECS: u64 = 1800;
pub const WIKI_CACHE_TTL_SECS: u64 = 3600;
pub const MAX_WIKI_CACHE_SIZE: usize = 30;
pub const MIN_CACHE_CONFIDENCE: f32 = 0.6;
pub const LLM_MEMO_TTL_SECS: u64 = 300;
pub const MAX_LLM_MEMO_SIZE: usize = 100;
pub const CMD_FAILURE_TTL_SECS: u64 = 3600;
pub const MAX_CMD_FAILURE_CACHE_SIZE: usize = 200;
pub const CMD_FAILURE_THRESHOLD: u32 = 3;
pub const INTENT_CACHE_TTL_SECS: u64 = 600;
pub const MAX_INTENT_CACHE_SIZE: usize = 50;

/// In-flight request tracking
pub struct InflightRequest {
    pub started_at: Instant,
}

/// Cached command failure
pub struct CommandFailure {
    pub error_type: String,
    pub failed_at: Instant,
}

/// Cached wiki search result
#[derive(Clone)]
pub struct CachedWikiResult {
    pub commands: Vec<String>,
    pub context: String,
    pub sources: Vec<String>,
    pub cached_at: Instant,
}

/// Cached command output with timestamp
pub struct CachedOutput {
    pub output: String,
    pub cached_at: Instant,
    pub is_static: bool,
}

/// Recipe book with TTL
pub struct CachedRecipeBook {
    pub book: RecipeBook,
    pub loaded_at: Instant,
}

/// Global command failure record
pub struct CommandFailureRecord {
    pub failure_count: u32,
    pub last_error_type: String,
    pub first_failed_at: Instant,
    pub last_failed_at: Instant,
}

/// Cached answer with metadata
pub struct CachedAnswer {
    pub answer: String,
    pub cached_at: Instant,
    pub confidence: f32,
}

/// Cached LLM response for memoization
pub struct CachedLlmResponse {
    pub response: String,
    pub cached_at: Instant,
}

/// Cached intent classification result
pub struct CachedIntent {
    pub interpreted_as: String,
    pub category: String,
    pub confidence: f32,
    pub topic: Option<String>,
    pub suggested_commands: Vec<String>,
    pub cached_at: Instant,
}
