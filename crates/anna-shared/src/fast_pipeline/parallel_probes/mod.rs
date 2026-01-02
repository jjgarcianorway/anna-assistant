//! Parallel Probe Engine (Part F) - v0.0.438.
//!
//! Execute probes in parallel with smart caching:
//! - Concurrency limit: 4 probes at once
//! - Cache TTL: 3 seconds
//! - Timeout per probe: 500ms
//!
//! Probe data should always be available fast.

pub mod cache;
pub mod engine;
pub mod types;

// Re-export commonly used items
pub use cache::{CacheEntry, CacheStats, ProbeCache};
pub use engine::ParallelProbeEngine;
pub use types::{
    BatchResult, ProbeBatch, ProbeResult, ProbeStatus, PreparedBatch,
    CACHE_TTL_SECONDS, MAX_CONCURRENT_PROBES, PROBE_TIMEOUT_MS,
};
