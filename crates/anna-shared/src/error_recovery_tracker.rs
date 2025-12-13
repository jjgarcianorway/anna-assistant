//! Error Recovery Tracker - Phase 97
//!
//! Tracks error recovery attempts and success rates.
//! Helps Anna learn which recovery strategies work best.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ErrorCategory {
    #[default]
    System,
    Network,
    Permission,
    NotFound,
    Timeout,
    Configuration,
    Dependency,
    Other,
}

impl ErrorCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::System => "System",
            ErrorCategory::Network => "Network",
            ErrorCategory::Permission => "Permission",
            ErrorCategory::NotFound => "Not Found",
            ErrorCategory::Timeout => "Timeout",
            ErrorCategory::Configuration => "Configuration",
            ErrorCategory::Dependency => "Dependency",
            ErrorCategory::Other => "Other",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ErrorCategory::System => "⚙",
            ErrorCategory::Network => "⚡",
            ErrorCategory::Permission => "🔒",
            ErrorCategory::NotFound => "?",
            ErrorCategory::Timeout => "⏱",
            ErrorCategory::Configuration => "⚙",
            ErrorCategory::Dependency => "→",
            ErrorCategory::Other => "·",
        }
    }
}

/// Recovery outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RecoveryOutcome {
    #[default]
    Success,
    PartialSuccess,
    Failed,
    Skipped,
    Manual,
}

impl RecoveryOutcome {
    pub fn name(&self) -> &'static str {
        match self {
            RecoveryOutcome::Success => "Success",
            RecoveryOutcome::PartialSuccess => "Partial",
            RecoveryOutcome::Failed => "Failed",
            RecoveryOutcome::Skipped => "Skipped",
            RecoveryOutcome::Manual => "Manual",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            RecoveryOutcome::Success => "✓",
            RecoveryOutcome::PartialSuccess => "~",
            RecoveryOutcome::Failed => "✗",
            RecoveryOutcome::Skipped => "-",
            RecoveryOutcome::Manual => "→",
        }
    }
}

/// An error recovery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryRecord {
    /// Error ID
    pub id: String,
    /// Error category
    pub category: ErrorCategory,
    /// Error message
    pub error_message: String,
    /// Recovery strategy used
    pub strategy: String,
    /// Recovery outcome
    pub outcome: RecoveryOutcome,
    /// Time taken (ms)
    pub duration_ms: u64,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Timestamp
    pub timestamp: u64,
}

/// Error recovery tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorRecoveryTracker {
    /// All recovery records
    pub records: Vec<ErrorRecoveryRecord>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Count by outcome
    pub by_outcome: HashMap<String, u64>,
    /// Strategy success rates
    pub strategy_success: HashMap<String, (u64, u64)>, // (successes, total)
    /// Total errors
    pub total_errors: u64,
    /// Total recovered
    pub total_recovered: u64,
}

impl ErrorRecoveryTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an error recovery attempt
    pub fn record(&mut self, record: ErrorRecoveryRecord) {
        *self.by_category.entry(record.category.name().to_string()).or_insert(0) += 1;
        *self.by_outcome.entry(record.outcome.name().to_string()).or_insert(0) += 1;

        // Update strategy stats
        let (successes, total) = self
            .strategy_success
            .entry(record.strategy.clone())
            .or_insert((0, 0));
        *total += 1;
        if record.outcome == RecoveryOutcome::Success {
            *successes += 1;
            self.total_recovered += 1;
        }

        self.total_errors += 1;
        self.records.push(record);
    }

    /// Get record by ID
    pub fn get(&self, id: &str) -> Option<&ErrorRecoveryRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get records by category
    pub fn by_err_category(&self, category: ErrorCategory) -> Vec<&ErrorRecoveryRecord> {
        self.records.iter().filter(|r| r.category == category).collect()
    }

    /// Get records by outcome
    pub fn by_rec_outcome(&self, outcome: RecoveryOutcome) -> Vec<&ErrorRecoveryRecord> {
        self.records.iter().filter(|r| r.outcome == outcome).collect()
    }

    /// Get successful recoveries
    pub fn successful(&self) -> Vec<&ErrorRecoveryRecord> {
        self.by_rec_outcome(RecoveryOutcome::Success)
    }

    /// Get failed recoveries
    pub fn failed(&self) -> Vec<&ErrorRecoveryRecord> {
        self.by_rec_outcome(RecoveryOutcome::Failed)
    }

    /// Get strategy success rate
    pub fn strategy_rate(&self, strategy: &str) -> Option<u8> {
        self.strategy_success.get(strategy).map(|(s, t)| {
            if *t == 0 {
                0
            } else {
                ((s * 100) / t) as u8
            }
        })
    }

    /// Get best strategies
    pub fn best_strategies(&self, limit: usize) -> Vec<(&String, u8)> {
        let mut rates: Vec<(&String, u8)> = self
            .strategy_success
            .iter()
            .map(|(s, (succ, total))| {
                let rate = if *total == 0 { 0 } else { ((succ * 100) / total) as u8 };
                (s, rate)
            })
            .collect();
        rates.sort_by(|a, b| b.1.cmp(&a.1));
        rates.into_iter().take(limit).collect()
    }

    /// Overall recovery rate
    pub fn recovery_rate(&self) -> u8 {
        if self.total_errors == 0 {
            0
        } else {
            ((self.total_recovered * 100) / self.total_errors) as u8
        }
    }

    /// Total record count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }
}

/// Format error recovery tracker for display
pub fn format_error_recovery_tracker(tracker: &ErrorRecoveryTracker) -> String {
    let mut lines = vec!["=== Error Recovery Tracker ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No error recovery records yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total errors: {}", tracker.total_errors));
    lines.push(format!("Total recovered: {}", tracker.total_recovered));
    lines.push(format!("Recovery rate: {}%", tracker.recovery_rate()));

    // By category
    if !tracker.by_category.is_empty() {
        lines.push(String::new());
        lines.push("By category:".to_string());
        for (cat, count) in &tracker.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    // Best strategies
    let best = tracker.best_strategies(5);
    if !best.is_empty() {
        lines.push(String::new());
        lines.push("Best strategies:".to_string());
        for (strategy, rate) in best {
            lines.push(format!("  {}: {}% success", strategy, rate));
        }
    }

    lines.join("\n")
}

/// Format error recovery tracker compact
pub fn format_error_recovery_tracker_compact(tracker: &ErrorRecoveryTracker) -> String {
    format!(
        "Errors: {} total | {} recovered | {}% rate",
        tracker.total_errors,
        tracker.total_recovered,
        tracker.recovery_rate()
    )
}

/// Format error recovery tracker one-line
pub fn format_error_recovery_tracker_oneline(tracker: &ErrorRecoveryTracker) -> String {
    format!(
        "{} errors ({}% recovered)",
        tracker.total_errors,
        tracker.recovery_rate()
    )
}

/// Check if query is about error recovery
pub fn is_error_recovery_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "error recovery",
        "recovery rate",
        "error handling",
        "how many errors",
        "failed recovery",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about error recovery
pub fn error_recovery_fun_fact(tracker: &ErrorRecoveryTracker) -> String {
    if tracker.records.is_empty() {
        return "No error recovery data yet!".to_string();
    }

    let facts = [
        format!("Anna has handled {} errors.", tracker.total_errors),
        format!("{}% of errors were successfully recovered.", tracker.recovery_rate()),
        format!("{} errors were recovered automatically.", tracker.total_recovered),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(id: &str, category: ErrorCategory, outcome: RecoveryOutcome) -> ErrorRecoveryRecord {
        ErrorRecoveryRecord {
            id: id.to_string(),
            category,
            error_message: "Test error".to_string(),
            strategy: "retry".to_string(),
            outcome,
            duration_ms: 100,
            retry_count: 1,
            timestamp: 1234567890,
        }
    }

    #[test]
    fn test_error_category() {
        assert_eq!(ErrorCategory::Network.name(), "Network");
        assert_eq!(ErrorCategory::Permission.symbol(), "🔒");
    }

    #[test]
    fn test_recovery_outcome() {
        assert_eq!(RecoveryOutcome::Success.name(), "Success");
        assert_eq!(RecoveryOutcome::Failed.symbol(), "✗");
    }

    #[test]
    fn test_record_error() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.total_errors, 1);
        assert_eq!(tracker.total_recovered, 1);
    }

    #[test]
    fn test_by_category() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));
        tracker.record(make_record("err2", ErrorCategory::Permission, RecoveryOutcome::Failed));

        assert_eq!(tracker.by_err_category(ErrorCategory::Network).len(), 1);
        assert_eq!(tracker.by_err_category(ErrorCategory::Permission).len(), 1);
    }

    #[test]
    fn test_by_outcome() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));
        tracker.record(make_record("err2", ErrorCategory::Network, RecoveryOutcome::Failed));

        assert_eq!(tracker.successful().len(), 1);
        assert_eq!(tracker.failed().len(), 1);
    }

    #[test]
    fn test_recovery_rate() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));
        tracker.record(make_record("err2", ErrorCategory::Network, RecoveryOutcome::Failed));

        assert_eq!(tracker.recovery_rate(), 50);
    }

    #[test]
    fn test_strategy_rate() {
        let mut tracker = ErrorRecoveryTracker::new();
        let mut rec = make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success);
        rec.strategy = "restart".to_string();
        tracker.record(rec);

        assert_eq!(tracker.strategy_rate("restart"), Some(100));
    }

    #[test]
    fn test_best_strategies() {
        let mut tracker = ErrorRecoveryTracker::new();
        let mut rec1 = make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success);
        rec1.strategy = "restart".to_string();
        tracker.record(rec1);

        let mut rec2 = make_record("err2", ErrorCategory::Network, RecoveryOutcome::Failed);
        rec2.strategy = "retry".to_string();
        tracker.record(rec2);

        let best = tracker.best_strategies(2);
        assert_eq!(best[0].0, "restart");
        assert_eq!(best[0].1, 100);
    }

    #[test]
    fn test_format_tracker() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));

        let output = format_error_recovery_tracker(&tracker);
        assert!(output.contains("Error Recovery Tracker"));
        assert!(output.contains("Total errors: 1"));
    }

    #[test]
    fn test_is_error_recovery_query() {
        assert!(is_error_recovery_query("show error recovery stats"));
        assert!(is_error_recovery_query("what is the recovery rate?"));
        assert!(!is_error_recovery_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));

        let fact = error_recovery_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
