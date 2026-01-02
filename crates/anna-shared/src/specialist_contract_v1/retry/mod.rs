//! Retry Strategy (Part C) - v0.0.440.
//!
//! For each specialist call:
//! - Timeout: 8s (local models are unreliable workers)
//! - Retries: 2 maximum
//! - Backoff: 250ms then 500ms
//!
//! Retry prompts:
//! 1) "You violated SRC v1. Output ONLY valid JSON matching schema. No prose."
//! 2) "Last chance. Output ONLY JSON. If uncertain, reduce scope and lower confidence."

pub mod config;
pub mod decision;
pub mod state;

// Re-export all public types for convenience
pub use config::{
    RetryConfig, BACKOFF_1_MS, BACKOFF_2_MS, MAX_RETRIES, REPAIR_PROMPT_1, REPAIR_PROMPT_2,
    SPECIALIST_TIMEOUT_MS,
};
pub use decision::{build_repair_prompt, RetryDecision, RetrySummary};
pub use state::{AttemptResult, RetryAttempt, RetryState};
