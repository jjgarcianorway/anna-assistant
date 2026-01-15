//! Probe Ledger - Per-request probe deduplication.
//!
//! Phase 22: Prevents duplicate probe execution within a single request.
//! Each request gets its own ledger to track which probes have been executed.

use std::collections::HashSet;

/// Per-request probe deduplication ledger.
/// Tracks which probes have been executed to prevent duplicates.
#[derive(Debug, Default)]
pub struct ProbeLedger {
    /// Set of executed probe commands (normalized).
    executed: HashSet<String>,
    /// Total probe execution count.
    total_count: usize,
    /// Count of skipped (duplicate) probes.
    skipped_count: usize,
}

impl ProbeLedger {
    /// Create a new empty probe ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a probe command should be executed.
    /// Returns true if the probe hasn't been executed yet.
    pub fn should_execute(&mut self, command: &str) -> bool {
        let normalized = normalize_command(command);
        if self.executed.contains(&normalized) {
            self.skipped_count += 1;
            false
        } else {
            self.executed.insert(normalized);
            self.total_count += 1;
            true
        }
    }

    /// Mark a probe as executed without checking.
    /// Use when the probe has already been run externally.
    pub fn mark_executed(&mut self, command: &str) {
        let normalized = normalize_command(command);
        if self.executed.insert(normalized) {
            self.total_count += 1;
        }
    }

    /// Get the number of unique probes executed.
    pub fn executed_count(&self) -> usize {
        self.total_count
    }

    /// Get the number of skipped (duplicate) probes.
    pub fn skipped_count(&self) -> usize {
        self.skipped_count
    }

    /// Check if a command has already been executed.
    pub fn was_executed(&self, command: &str) -> bool {
        let normalized = normalize_command(command);
        self.executed.contains(&normalized)
    }

    /// Get all executed commands.
    pub fn executed_commands(&self) -> Vec<&str> {
        self.executed.iter().map(|s| s.as_str()).collect()
    }

    /// Reset the ledger (for testing or new request).
    pub fn reset(&mut self) {
        self.executed.clear();
        self.total_count = 0;
        self.skipped_count = 0;
    }
}

/// Normalize a command for comparison.
/// Removes extra whitespace and normalizes paths.
fn normalize_command(cmd: &str) -> String {
    // Collapse whitespace
    let normalized: String = cmd.split_whitespace().collect::<Vec<_>>().join(" ");
    // Lowercase for case-insensitive comparison
    normalized.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_deduplication() {
        let mut ledger = ProbeLedger::new();

        // First execution should succeed
        assert!(ledger.should_execute("df -h"));
        assert_eq!(ledger.executed_count(), 1);

        // Duplicate should be skipped
        assert!(!ledger.should_execute("df -h"));
        assert_eq!(ledger.executed_count(), 1);
        assert_eq!(ledger.skipped_count(), 1);

        // Different command should execute
        assert!(ledger.should_execute("free -h"));
        assert_eq!(ledger.executed_count(), 2);
    }

    #[test]
    fn test_whitespace_normalization() {
        let mut ledger = ProbeLedger::new();

        // Execute with extra whitespace
        assert!(ledger.should_execute("df   -h"));
        // Same command with different whitespace should be duplicate
        assert!(!ledger.should_execute("df -h"));
        assert_eq!(ledger.executed_count(), 1);
        assert_eq!(ledger.skipped_count(), 1);
    }

    #[test]
    fn test_case_insensitive() {
        let mut ledger = ProbeLedger::new();

        assert!(ledger.should_execute("DF -H"));
        assert!(!ledger.should_execute("df -h"));
        assert_eq!(ledger.skipped_count(), 1);
    }

    #[test]
    fn test_mark_executed() {
        let mut ledger = ProbeLedger::new();

        ledger.mark_executed("df -h");
        assert!(ledger.was_executed("df -h"));
        assert!(!ledger.should_execute("df -h"));
        assert_eq!(ledger.executed_count(), 1);
        assert_eq!(ledger.skipped_count(), 1);
    }

    #[test]
    fn test_reset() {
        let mut ledger = ProbeLedger::new();

        ledger.should_execute("df -h");
        ledger.should_execute("free -h");
        ledger.reset();

        assert_eq!(ledger.executed_count(), 0);
        assert!(ledger.should_execute("df -h"));
    }
}
