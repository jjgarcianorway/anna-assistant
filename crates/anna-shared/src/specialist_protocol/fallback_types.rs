//! Types for fallback handling (v0.0.428).

use std::collections::HashMap;

/// Fallback context: what we know when a specialist fails
#[derive(Debug, Clone)]
pub struct FallbackContext {
    /// Ticket ID
    pub ticket_id: String,
    /// Domain of the query
    pub domain: String,
    /// Intent of the query
    pub intent: String,
    /// Original user question
    pub question: String,
    /// Probe results we have (probe_id -> output)
    pub probe_results: HashMap<String, String>,
    /// Why fallback was triggered
    pub reason: FallbackReason,
    /// Elapsed time before failure (ms)
    pub elapsed_ms: u64,
}

/// Why we're using fallback
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackReason {
    /// LLM call timed out
    Timeout,
    /// JSON parsing failed
    ParseError(String),
    /// Validation failed
    ValidationFailed(String),
    /// LLM returned error status
    LlmError(String),
    /// No specialist available
    NoSpecialist,
    /// Retry limit exceeded
    RetryExhausted,
}

impl std::fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "timeout"),
            Self::ParseError(e) => write!(f, "parse_error: {}", e),
            Self::ValidationFailed(e) => write!(f, "validation_failed: {}", e),
            Self::LlmError(e) => write!(f, "llm_error: {}", e),
            Self::NoSpecialist => write!(f, "no_specialist"),
            Self::RetryExhausted => write!(f, "retry_exhausted"),
        }
    }
}

/// A fact extracted from a probe
#[derive(Debug, Clone)]
pub struct ExtractedFact {
    pub probe_id: String,
    pub summary: String,
    pub raw_snippet: String,
}

/// User-facing error message (never shows internal details)
pub fn user_friendly_error_message(reason: &FallbackReason) -> &'static str {
    match reason {
        FallbackReason::Timeout => "My analysis is taking longer than expected.",
        FallbackReason::ParseError(_) => "I had trouble processing this request.",
        FallbackReason::ValidationFailed(_) => "I couldn't verify my response.",
        FallbackReason::LlmError(_) => "I encountered an issue during analysis.",
        FallbackReason::NoSpecialist => "I don't have a specialist for this topic.",
        FallbackReason::RetryExhausted => "I couldn't complete this request.",
    }
}

/// Debug-mode error message (shows internal details)
pub fn debug_error_message(reason: &FallbackReason) -> String {
    match reason {
        FallbackReason::Timeout => "Specialist LLM call timed out".to_string(),
        FallbackReason::ParseError(e) => format!("JSON parse error: {}", e),
        FallbackReason::ValidationFailed(e) => format!("Validation failed: {}", e),
        FallbackReason::LlmError(e) => format!("LLM error: {}", e),
        FallbackReason::NoSpecialist => "No specialist registered for domain".to_string(),
        FallbackReason::RetryExhausted => "Max retries exceeded".to_string(),
    }
}

/// Truncate string to max length
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
