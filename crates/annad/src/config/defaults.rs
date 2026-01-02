//! Default configuration values for annad.
//!
//! All default values are centralized here for maintainability.

// ============================================================================
// LLM Model Defaults
// ============================================================================

pub fn translator_model() -> String {
    // v0.0.397: Translator needs 3B+ for reliable JSON output
    // 0.5B models produce garbage JSON causing routing failures
    "qwen2.5:3b-instruct".to_string()
}

pub fn junior_model() -> String {
    // v0.0.277: Junior uses 3b model - smarter than translator, faster than senior
    "qwen2.5:3b-instruct".to_string()
}

pub fn senior_model() -> String {
    // v0.0.277: Senior uses 7b model - smartest, for complex queries
    "qwen2.5:7b-instruct".to_string()
}

pub fn specialist_model() -> String {
    // Legacy: maps to junior model for backwards compatibility
    junior_model()
}

pub fn supervisor_model() -> String {
    // v0.0.397: Supervisor needs 3B+ like translator for reliable JSON
    "qwen2.5:3b-instruct".to_string()
}

// ============================================================================
// LLM Timeout Defaults
// ============================================================================

pub fn translator_timeout() -> u64 {
    // v0.0.398: Increased to 10s for 3B+ models (slower but reliable)
    10
}

pub fn specialist_timeout() -> u64 {
    12 // v0.0.140: increased from 6 - give LLM proper time to respond
}

pub fn max_specialist_prompt() -> usize {
    16_384 // 16KB cap to prevent slow inference
}

pub fn supervisor_timeout() -> u64 {
    6
}

pub fn probe_timeout() -> u64 {
    4
}

pub fn probes_total_timeout() -> u64 {
    10
}

// ============================================================================
// Budget Defaults (METER phase)
// ============================================================================

pub fn translator_budget() -> u64 {
    6_000 // v0.0.140: 6s - allow retries (5s timeout + retry delays)
}

pub fn probes_budget() -> u64 {
    10_000 // v0.0.140: 10s - reasonable probe window
}

pub fn specialist_budget() -> u64 {
    15_000 // v0.0.140: 15s - give LLM proper time with retries
}

pub fn supervisor_budget() -> u64 {
    6_000 // v0.0.140: 6s - review gate with buffer
}

pub fn total_budget() -> u64 {
    35_000 // v0.0.140: 35s total - reliability over speed
}

pub fn margin_budget() -> u64 {
    1_000 // 1 second
}

// ============================================================================
// Daemon Defaults
// ============================================================================

pub fn fast_path_enabled() -> bool {
    true // Fast path enabled by default
}

pub fn fast_path_fallback() -> bool {
    true // Fallback to fast path on translator timeout
}

pub fn debug_mode() -> bool {
    true // Debug ON by default
}

pub fn auto_update() -> bool {
    true
}

pub fn update_interval() -> u64 {
    600
}

pub fn request_timeout() -> u64 {
    40 // v0.0.140: 40s total - allow LLM retries and proper inference time
}

pub fn snapshot_max_age() -> u64 {
    300 // 5 minutes - v0.0.36: snapshot freshness window
}
