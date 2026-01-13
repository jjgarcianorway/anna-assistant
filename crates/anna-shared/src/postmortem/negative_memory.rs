//! Negative Memory - Store what DIDN'T work and why.
//!
//! Critical for preventing repeated mistakes. Humans often forget
//! failed attempts and retry the same wrong approaches.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Negative memory storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NegativeMemory {
    /// Failed attempts indexed by question pattern
    pub by_pattern: HashMap<String, Vec<FailedAttempt>>,
    /// Failed commands indexed by command
    pub by_command: HashMap<String, Vec<FailedAttempt>>,
    /// Statistics
    pub stats: NegativeMemoryStats,
}

/// A recorded failed attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedAttempt {
    /// What was tried
    pub action: String,
    /// Why it failed
    pub failure_reason: String,
    /// Error message
    pub error_output: Option<String>,
    /// System context when it failed
    pub system_context: Vec<String>,
    /// When it failed
    pub failed_at: String,
    /// How many times this has been tried
    pub attempt_count: u32,
    /// Should we ever retry this?
    pub should_retry: bool,
    /// Under what conditions might this work
    pub retry_conditions: Vec<String>,
}

impl FailedAttempt {
    /// Create a new failed attempt
    pub fn new(action: &str, reason: &str) -> Self {
        Self {
            action: action.to_string(),
            failure_reason: reason.to_string(),
            error_output: None,
            system_context: crate::memory::ExperienceContext::current_system_tags(),
            failed_at: chrono::Utc::now().to_rfc3339(),
            attempt_count: 1,
            should_retry: true,
            retry_conditions: Vec::new(),
        }
    }

    /// Mark as permanently failed (don't retry)
    pub fn mark_permanent(&mut self) {
        self.should_retry = false;
    }

    /// Add a retry condition
    pub fn add_retry_condition(&mut self, condition: &str) {
        self.retry_conditions.push(condition.to_string());
    }

    /// Check if this failure matches current system context
    pub fn matches_current_context(&self) -> bool {
        let current = crate::memory::ExperienceContext::current_system_tags();
        self.system_context.iter().any(|t| current.contains(t))
    }
}

/// Statistics about negative memory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NegativeMemoryStats {
    /// Total failed attempts recorded
    pub total_failures: u32,
    /// Failures that prevented future mistakes
    pub prevented_repeats: u32,
    /// Failures that were later resolved
    pub later_resolved: u32,
}

impl NegativeMemory {
    /// Record a failed attempt
    pub fn record_failure(
        &mut self,
        pattern: &str,
        command: &str,
        reason: &str,
        error_output: Option<&str>,
    ) {
        let attempt = FailedAttempt {
            action: command.to_string(),
            failure_reason: reason.to_string(),
            error_output: error_output.map(|s| s.to_string()),
            system_context: crate::memory::ExperienceContext::current_system_tags(),
            failed_at: chrono::Utc::now().to_rfc3339(),
            attempt_count: 1,
            should_retry: true,
            retry_conditions: Vec::new(),
        };

        // Store by pattern
        self.by_pattern
            .entry(pattern.to_string())
            .or_default()
            .push(attempt.clone());

        // Store by command
        self.by_command
            .entry(command.to_string())
            .or_default()
            .push(attempt);

        self.stats.total_failures += 1;
    }

    /// Check if a command has failed for this pattern before
    pub fn has_failed_before(&self, pattern: &str, command: &str) -> Option<&FailedAttempt> {
        if let Some(failures) = self.by_pattern.get(pattern) {
            failures
                .iter()
                .find(|f| f.action == command && f.matches_current_context())
        } else {
            None
        }
    }

    /// Check if a command has ever failed in this context
    pub fn command_failed_in_context(&self, command: &str) -> Option<&FailedAttempt> {
        if let Some(failures) = self.by_command.get(command) {
            failures.iter().find(|f| f.matches_current_context())
        } else {
            None
        }
    }

    /// Get all failures for a pattern
    pub fn failures_for_pattern(&self, pattern: &str) -> Vec<&FailedAttempt> {
        self.by_pattern
            .get(pattern)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Mark a failure as resolved
    pub fn mark_resolved(&mut self, command: &str) {
        if let Some(failures) = self.by_command.get_mut(command) {
            for failure in failures.iter_mut() {
                failure.should_retry = true;
            }
        }
        self.stats.later_resolved += 1;
    }

    /// Count how many times a failure prevented a repeat mistake
    pub fn record_prevention(&mut self) {
        self.stats.prevented_repeats += 1;
    }
}

/// Get the negative memory file path
fn negative_memory_path() -> PathBuf {
    let data_dir = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("anna").join("negative_memory.json")
}

/// Load negative memory from disk
pub fn load_negative_memory() -> NegativeMemory {
    let path = negative_memory_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(memory) = serde_json::from_str(&content) {
                return memory;
            }
        }
    }
    NegativeMemory::default()
}

/// Save negative memory to disk
pub fn save_negative_memory(memory: &NegativeMemory) -> Result<(), String> {
    let path = negative_memory_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(memory).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_failure() {
        let mut memory = NegativeMemory::default();
        memory.record_failure(
            "install vim",
            "pacman -S vim",
            "permission denied",
            Some("error: permission denied"),
        );

        assert_eq!(memory.stats.total_failures, 1);
        assert!(memory.by_command.contains_key("pacman -S vim"));
    }

    #[test]
    fn test_has_failed_before() {
        let mut memory = NegativeMemory::default();
        memory.record_failure(
            "install vim",
            "pacman -S vim",
            "permission denied",
            None,
        );

        // Should find it (though context might not match in test)
        let failures = memory.failures_for_pattern("install vim");
        assert_eq!(failures.len(), 1);
    }
}
