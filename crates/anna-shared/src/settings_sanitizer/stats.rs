// v0.0.643: Settings Sanitizer Stats (Phase 219)
// Statistics tracking for sanitization operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::SanitizationType;

/// Sanitizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SanitizerStats {
    /// Total sanitized
    pub total_sanitized: usize,
    /// Changed count
    pub changed: usize,
    /// Unchanged count
    pub unchanged: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SanitizerStats {
    /// Record sanitization
    pub fn record(&mut self, sanitization_type: SanitizationType, changed: bool) {
        self.total_sanitized += 1;
        if changed {
            self.changed += 1;
        } else {
            self.unchanged += 1;
        }
        *self.by_type.entry(sanitization_type.to_string()).or_insert(0) += 1;
    }

    /// Change rate
    pub fn change_rate(&self) -> f64 {
        if self.total_sanitized == 0 {
            0.0
        } else {
            self.changed as f64 / self.total_sanitized as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = SanitizerStats::default();
        s.record(SanitizationType::Trim, true);
        s.record(SanitizationType::Trim, false);
        assert_eq!(s.total_sanitized, 2);
        assert_eq!(s.changed, 1);
    }
}
