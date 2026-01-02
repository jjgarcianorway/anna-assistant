//! Retry configuration and constants.

use std::time::Duration;

/// Specialist call timeout in milliseconds.
pub const SPECIALIST_TIMEOUT_MS: u64 = 8000;

/// Maximum retry attempts.
pub const MAX_RETRIES: usize = 2;

/// First backoff duration in milliseconds.
pub const BACKOFF_1_MS: u64 = 250;

/// Second backoff duration in milliseconds.
pub const BACKOFF_2_MS: u64 = 500;

/// Retry prompt for first retry.
pub const REPAIR_PROMPT_1: &str =
    "You violated SRC v1. Output ONLY valid JSON matching schema. No prose. \
    Required fields: case_id, department, assessment (summary, confidence, risk). \
    No markdown. No extra text.";

/// Retry prompt for second retry.
pub const REPAIR_PROMPT_2: &str =
    "Last chance. Output ONLY JSON. If uncertain, reduce scope and lower confidence. \
    Example: {\"case_id\":\"...\",\"department\":\"Performance\",\
    \"assessment\":{\"summary\":\"Brief answer.\",\"confidence\":0.6,\"risk\":\"read_only\"},\
    \"actions\":[],\"citations\":[]}";

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Timeout per call in milliseconds.
    pub timeout_ms: u64,
    /// Maximum retry attempts.
    pub max_retries: usize,
    /// Backoff durations.
    pub backoffs: Vec<Duration>,
    /// Repair prompts.
    pub repair_prompts: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            timeout_ms: SPECIALIST_TIMEOUT_MS,
            max_retries: MAX_RETRIES,
            backoffs: vec![
                Duration::from_millis(BACKOFF_1_MS),
                Duration::from_millis(BACKOFF_2_MS),
            ],
            repair_prompts: vec![REPAIR_PROMPT_1.to_string(), REPAIR_PROMPT_2.to_string()],
        }
    }
}

impl RetryConfig {
    /// Get timeout as Duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// Get backoff for attempt (0-indexed).
    pub fn backoff_for_attempt(&self, attempt: usize) -> Duration {
        self.backoffs
            .get(attempt)
            .cloned()
            .unwrap_or(Duration::from_millis(500))
    }

    /// Get repair prompt for attempt (0-indexed).
    pub fn repair_prompt_for_attempt(&self, attempt: usize) -> &str {
        self.repair_prompts
            .get(attempt)
            .map(|s| s.as_str())
            .unwrap_or(REPAIR_PROMPT_2)
    }

    /// Check if can retry.
    pub fn can_retry(&self, attempt: usize) -> bool {
        attempt < self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.timeout_ms, 8000);
        assert_eq!(config.max_retries, 2);
    }
}
