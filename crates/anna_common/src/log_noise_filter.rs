//! v6.39.0: Log Noise Filter - Intelligent Hardware Error Filtering
//!
//! Filters benign hardware errors from critical log analysis to reduce
//! false-positive CRITICAL status alerts.
//!
//! Philosophy:
//! - Gaming controllers (DualSense, Xbox) produce harmless CRC errors
//! - Repetitive identical errors (>100x) are likely hardware noise, not system issues
//! - Real system errors must still escalate to CRITICAL
//! - Transparency: Show what was filtered and why

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity level for log entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A pattern that matches benign hardware noise
#[derive(Debug, Clone)]
pub struct NoisePattern {
    /// The regex pattern to match against log messages
    pub pattern: Regex,
    /// Human-readable reason for filtering
    pub reason: &'static str,
    /// Override severity (Some = downgrade to this level, None = suppress entirely)
    pub severity_override: Option<Severity>,
}

impl NoisePattern {
    /// Create a new noise pattern from a regex string
    fn new(pattern: &str, reason: &'static str, severity_override: Option<Severity>) -> Self {
        Self {
            pattern: Regex::new(pattern).expect("Invalid regex pattern"),
            reason,
            severity_override,
        }
    }
}

/// Filter for benign hardware errors in logs
pub struct LogNoiseFilter {
    patterns: Vec<NoisePattern>,
}

impl LogNoiseFilter {
    /// Create a new log noise filter with default patterns
    pub fn new() -> Self {
        let patterns = vec![
            // PlayStation DualSense controller CRC errors
            NoisePattern::new(
                r"DualSense input CRC's check failed",
                "PlayStation controller hardware noise",
                Some(Severity::Info),
            ),
            NoisePattern::new(
                r"playstation.*CRC.*failed",
                "PlayStation controller communication errors",
                Some(Severity::Info),
            ),
            // Xbox controller disconnects (common during low battery)
            NoisePattern::new(
                r"xpad.*input/input\d+.*disconnected",
                "Xbox controller disconnection (low battery)",
                Some(Severity::Info),
            ),
            // Generic USB device enumeration errors (transient)
            NoisePattern::new(
                r"usb.*device descriptor read.*error",
                "Transient USB enumeration error",
                Some(Severity::Info),
            ),
            // Bluetooth controller pairing noise
            NoisePattern::new(
                r"bluetooth.*hci0.*LE connection failed",
                "Bluetooth controller connection attempt",
                Some(Severity::Info),
            ),
        ];

        Self { patterns }
    }

    /// Check if a log message should be filtered or downgraded
    ///
    /// Returns:
    /// - `Some(&NoisePattern)` if the message matches a noise pattern
    /// - `None` if the message is NOT noise and should be treated as a real error
    pub fn should_filter(&self, log_message: &str) -> Option<&NoisePattern> {
        for pattern in &self.patterns {
            if pattern.pattern.is_match(log_message) {
                return Some(pattern);
            }
        }
        None
    }

    /// Check if a set of log entries is repetitive noise
    ///
    /// Definition of repetitive noise:
    /// - The same error message appears >100 times
    /// - It represents >80% of all errors
    /// - This indicates hardware spam, not a system health issue
    pub fn is_repetitive_noise(&self, log_messages: &[&str]) -> bool {
        if log_messages.len() < 100 {
            return false; // Not enough data to determine if it's noise
        }

        // Count occurrences of each unique message
        let mut message_counts: HashMap<&str, usize> = HashMap::new();
        for msg in log_messages {
            *message_counts.entry(msg).or_insert(0) += 1;
        }

        // Find the most common message
        if let Some((_, max_count)) = message_counts.iter().max_by_key(|(_, count)| *count) {
            let repetition_ratio = *max_count as f64 / log_messages.len() as f64;

            // If one message is >80% of all errors, it's noise
            if repetition_ratio > 0.8 && *max_count > 100 {
                return true;
            }
        }

        false
    }

    /// Filter a list of log messages, returning (real_errors, filtered_errors)
    ///
    /// Real errors: Should escalate to CRITICAL status
    /// Filtered errors: Benign hardware noise that should not affect health status
    pub fn filter_logs<'a>(&self, log_messages: &[&'a str]) -> (Vec<&'a str>, Vec<(&'a str, &str)>) {
        let mut real_errors = Vec::new();
        let mut filtered_errors = Vec::new();

        for msg in log_messages {
            if let Some(pattern) = self.should_filter(msg) {
                // This is noise - add to filtered list with reason
                filtered_errors.push((*msg, pattern.reason));
            } else {
                // This is a real error
                real_errors.push(*msg);
            }
        }

        (real_errors, filtered_errors)
    }

    /// Get a summary of why errors were filtered
    ///
    /// Returns a human-readable explanation like:
    /// "10 hardware errors filtered - PlayStation controller noise"
    pub fn filter_summary(&self, filtered_count: usize, reason: &str) -> String {
        if filtered_count == 0 {
            return String::new();
        }

        if filtered_count == 1 {
            format!("1 hardware error filtered - {}", reason)
        } else {
            format!("{} hardware errors filtered - {}", filtered_count, reason)
        }
    }
}

impl Default for LogNoiseFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dualsense_pattern_matches() {
        let filter = LogNoiseFilter::new();

        let dualsense_error = "playstation 0005:054C:0CE6.0010: DualSense input CRC's check failed";
        let pattern = filter.should_filter(dualsense_error);

        assert!(pattern.is_some(), "DualSense error should be filtered");
        assert_eq!(pattern.unwrap().reason, "PlayStation controller hardware noise");
        assert_eq!(pattern.unwrap().severity_override, Some(Severity::Info));
    }

    #[test]
    fn test_non_matching_errors_pass_through() {
        let filter = LogNoiseFilter::new();

        let real_error = "kernel: Out of memory: Killed process 1234";
        let pattern = filter.should_filter(real_error);

        assert!(pattern.is_none(), "Real system errors should NOT be filtered");
    }

    #[test]
    fn test_repetitive_detection_works() {
        let filter = LogNoiseFilter::new();

        // Create 150 identical messages (90% repetition)
        let mut messages = vec!["Same error message"; 135];
        messages.extend(vec!["Different error 1", "Different error 2", "Different error 3"]);
        messages.extend(vec!["Another error"; 12]); // Total 150

        assert!(filter.is_repetitive_noise(&messages), "Should detect repetitive noise (>80% same message)");
    }

    #[test]
    fn test_repetitive_detection_diverse_errors() {
        let filter = LogNoiseFilter::new();

        // Create diverse errors (no single message dominates)
        let messages = vec![
            "Error A", "Error B", "Error C", "Error D", "Error E",
            "Error A", "Error B", "Error C", "Error D", "Error E",
        ];

        assert!(!filter.is_repetitive_noise(&messages), "Diverse errors should NOT be noise");
    }

    #[test]
    fn test_filter_logs_separates_real_from_noise() {
        let filter = LogNoiseFilter::new();

        let logs = vec![
            "playstation 0005:054C:0CE6.0010: DualSense input CRC's check failed",
            "kernel: Out of memory: Killed process 1234",
            "playstation 0005:054C:0CE6.0011: DualSense input CRC's check failed",
            "ext4: I/O error writing to inode",
        ];

        let (real_errors, filtered_errors) = filter.filter_logs(&logs);

        assert_eq!(real_errors.len(), 2, "Should have 2 real errors");
        assert_eq!(filtered_errors.len(), 2, "Should have 2 filtered errors");

        assert!(real_errors.contains(&"kernel: Out of memory: Killed process 1234"));
        assert!(real_errors.contains(&"ext4: I/O error writing to inode"));
    }

    #[test]
    fn test_filter_summary_formatting() {
        let filter = LogNoiseFilter::new();

        let summary_zero = filter.filter_summary(0, "test");
        assert_eq!(summary_zero, "", "Zero filtered should return empty string");

        let summary_one = filter.filter_summary(1, "PlayStation controller noise");
        assert_eq!(summary_one, "1 hardware error filtered - PlayStation controller noise");

        let summary_many = filter.filter_summary(10, "PlayStation controller noise");
        assert_eq!(summary_many, "10 hardware errors filtered - PlayStation controller noise");
    }

    #[test]
    fn test_xbox_controller_pattern() {
        let filter = LogNoiseFilter::new();

        let xbox_error = "xpad 3-1:1.0: input/input42 disconnected";
        let pattern = filter.should_filter(xbox_error);

        assert!(pattern.is_some(), "Xbox controller error should be filtered");
        assert_eq!(pattern.unwrap().reason, "Xbox controller disconnection (low battery)");
    }

    #[test]
    fn test_usb_transient_errors() {
        let filter = LogNoiseFilter::new();

        let usb_error = "usb 1-2: device descriptor read/64, error -32";
        let pattern = filter.should_filter(usb_error);

        assert!(pattern.is_some(), "Transient USB error should be filtered");
        assert_eq!(pattern.unwrap().reason, "Transient USB enumeration error");
    }

    #[test]
    fn test_bluetooth_controller_errors() {
        let filter = LogNoiseFilter::new();

        let bt_error = "bluetooth hci0: LE connection failed";
        let pattern = filter.should_filter(bt_error);

        assert!(pattern.is_some(), "Bluetooth connection error should be filtered");
        assert_eq!(pattern.unwrap().reason, "Bluetooth controller connection attempt");
    }

    #[test]
    fn test_severity_override_applied() {
        let filter = LogNoiseFilter::new();

        let dualsense_error = "playstation 0005:054C:0CE6.0010: DualSense input CRC's check failed";
        let pattern = filter.should_filter(dualsense_error).expect("Should match pattern");

        assert_eq!(pattern.severity_override, Some(Severity::Info), "Should downgrade to Info");
    }

    #[test]
    fn test_repetitive_noise_threshold() {
        let filter = LogNoiseFilter::new();

        // Test threshold: Need >100 identical messages at >80% ratio
        // First test: 100 identical at 80% - should NOT be noise (need >100 count)
        let mut messages = vec!["Same error"; 100];
        messages.extend(vec!["Different error"; 25]); // Total 125, 80% same (100/125)

        assert!(!filter.is_repetitive_noise(&messages), "Should require >100 occurrences (not ==100)");

        // Add one more identical message to cross threshold
        messages.push("Same error"); // Now 101/126 = 80.2%
        assert!(filter.is_repetitive_noise(&messages), "Should detect noise with >100 identical at >80%");
    }

    #[test]
    fn test_empty_log_list() {
        let filter = LogNoiseFilter::new();

        let empty_logs: Vec<&str> = vec![];
        let (real_errors, filtered_errors) = filter.filter_logs(&empty_logs);

        assert_eq!(real_errors.len(), 0);
        assert_eq!(filtered_errors.len(), 0);
    }

    #[test]
    fn test_all_filtered_no_real_errors() {
        let filter = LogNoiseFilter::new();

        let logs = vec![
            "playstation 0005:054C:0CE6.0010: DualSense input CRC's check failed",
            "playstation 0005:054C:0CE6.0011: DualSense input CRC's check failed",
            "xpad 3-1:1.0: input/input42 disconnected",
        ];

        let (real_errors, filtered_errors) = filter.filter_logs(&logs);

        assert_eq!(real_errors.len(), 0, "All errors should be filtered");
        assert_eq!(filtered_errors.len(), 3, "Should have 3 filtered errors");
    }
}
