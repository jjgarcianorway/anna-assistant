//! Recipe usage statistics (v0.0.420).

use serde::{Deserialize, Serialize};

/// Statistics for recipe usage and success tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeStats {
    /// Total times this recipe was used
    #[serde(default)]
    pub times_used: u64,
    /// Number of successful executions
    #[serde(default)]
    pub success_count: u64,
    /// Number of failed executions
    #[serde(default)]
    pub failure_count: u64,
    /// Unix timestamp of last use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<u64>,
    /// Average execution duration in milliseconds
    #[serde(default)]
    pub avg_duration_ms: u64,
}

impl RecipeStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful execution
    pub fn record_success(&mut self, duration_ms: u64) {
        self.times_used += 1;
        self.success_count += 1;
        self.last_used_at = Some(current_timestamp());
        self.update_avg_duration(duration_ms);
    }

    /// Record a failed execution
    pub fn record_failure(&mut self, duration_ms: u64) {
        self.times_used += 1;
        self.failure_count += 1;
        self.last_used_at = Some(current_timestamp());
        self.update_avg_duration(duration_ms);
    }

    /// Update average duration (exponential moving average)
    fn update_avg_duration(&mut self, duration_ms: u64) {
        if self.times_used == 1 {
            self.avg_duration_ms = duration_ms;
        } else {
            // EMA with alpha=0.2 to smooth out outliers
            self.avg_duration_ms =
                (self.avg_duration_ms as f64 * 0.8 + duration_ms as f64 * 0.2) as u64;
        }
    }

    /// Calculate success rate (0.0 - 1.0)
    pub fn success_rate(&self) -> f32 {
        if self.times_used == 0 {
            1.0 // New recipes start with 100% (benefit of doubt)
        } else {
            self.success_count as f32 / self.times_used as f32
        }
    }

    /// Check if recipe is reliable (>= 80% success rate with at least 2 uses)
    pub fn is_reliable(&self) -> bool {
        if self.times_used < 2 {
            true // New recipes are assumed reliable
        } else {
            self.success_rate() >= 0.80
        }
    }

    /// Check if recipe is mature (used 3+ times successfully)
    pub fn is_mature(&self) -> bool {
        self.success_count >= 3
    }

    /// Check if recipe should be disabled (too many failures)
    pub fn should_disable(&self) -> bool {
        // Disable if: >= 5 uses AND < 50% success rate
        self.times_used >= 5 && self.success_rate() < 0.50
    }

    /// Get confidence multiplier based on maturity
    pub fn maturity_multiplier(&self) -> f32 {
        match self.success_count {
            0 => 0.7,       // Untested
            1..=2 => 0.8,   // New
            3..=5 => 0.9,   // Young
            6..=10 => 0.95, // Maturing
            _ => 1.0,       // Mature
        }
    }

    /// Days since last use (None if never used)
    pub fn days_since_last_use(&self) -> Option<u64> {
        self.last_used_at.map(|last| {
            let now = current_timestamp();
            (now.saturating_sub(last)) / 86400
        })
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_rate() {
        let mut stats = RecipeStats::new();
        assert_eq!(stats.success_rate(), 1.0);

        stats.record_success(100);
        assert_eq!(stats.success_rate(), 1.0);

        stats.record_failure(100);
        assert_eq!(stats.success_rate(), 0.5);
    }

    #[test]
    fn test_maturity() {
        let mut stats = RecipeStats::new();
        assert!(!stats.is_mature());

        stats.record_success(100);
        stats.record_success(100);
        assert!(!stats.is_mature());

        stats.record_success(100);
        assert!(stats.is_mature());
    }

    #[test]
    fn test_should_disable() {
        let mut stats = RecipeStats::new();

        // Record 3 failures and 2 successes (40% success)
        for _ in 0..3 {
            stats.record_failure(100);
        }
        for _ in 0..2 {
            stats.record_success(100);
        }

        assert!(stats.should_disable());
    }
}
