//! Recipe statistics types (v0.0.423).
//!
//! Usage tracking and statistics for recipes.

use serde::{Deserialize, Serialize};

/// Recipe usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeStats {
    /// Total times matched
    pub times_matched: u32,
    /// Times executed
    pub times_executed: u32,
    /// Successful executions
    pub times_succeeded: u32,
    /// Failed executions
    pub times_failed: u32,
    /// Times user skipped/rejected
    pub times_skipped: u32,
    /// Last used timestamp (Unix epoch seconds)
    pub last_used: Option<u64>,
    /// Average execution time in ms
    pub avg_execution_ms: u64,
}

impl RecipeStats {
    /// Success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f32 {
        if self.times_executed == 0 {
            0.0
        } else {
            self.times_succeeded as f32 / self.times_executed as f32
        }
    }

    /// Whether recipe is mature (enough usage data)
    pub fn is_mature(&self) -> bool {
        self.times_executed >= super::MIN_MATURE_USES
    }

    /// Record a match
    pub fn record_match(&mut self) {
        self.times_matched += 1;
        self.last_used = Some(now_epoch());
    }

    /// Record successful execution
    pub fn record_success(&mut self, execution_ms: u64) {
        self.times_executed += 1;
        self.times_succeeded += 1;
        self.update_avg_time(execution_ms);
    }

    /// Record failed execution
    pub fn record_failure(&mut self) {
        self.times_executed += 1;
        self.times_failed += 1;
    }

    /// Record user skip
    pub fn record_skip(&mut self) {
        self.times_skipped += 1;
    }

    fn update_avg_time(&mut self, new_time: u64) {
        if self.avg_execution_ms == 0 {
            self.avg_execution_ms = new_time;
        } else {
            // Rolling average
            self.avg_execution_ms = (self.avg_execution_ms * 3 + new_time) / 4;
        }
    }
}

/// Get current Unix epoch seconds
fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_stats() {
        let mut stats = RecipeStats::default();
        stats.record_success(100);
        stats.record_success(200);
        assert_eq!(stats.times_executed, 2);
        assert_eq!(stats.success_rate(), 1.0);

        stats.record_failure();
        assert!(stats.success_rate() < 1.0);
    }
}
