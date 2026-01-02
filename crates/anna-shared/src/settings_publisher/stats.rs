// v0.0.634: Publisher Statistics (Phase 210)
// Statistics tracking for settings publisher

use serde::{Deserialize, Serialize};

/// Publisher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublisherStats {
    /// Total published
    pub total_published: usize,
    /// Successful publishes
    pub successful: usize,
    /// Failed publishes
    pub failed: usize,
    /// Buffered events
    pub buffered: usize,
}

impl PublisherStats {
    /// Record success
    pub fn record_success(&mut self) {
        self.total_published += 1;
        self.successful += 1;
    }

    /// Record failure
    pub fn record_failure(&mut self) {
        self.total_published += 1;
        self.failed += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_published == 0 {
            1.0
        } else {
            self.successful as f64 / self.total_published as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = PublisherStats::default();
        s.record_success();
        s.record_failure();
        assert_eq!(s.total_published, 2);
    }
}
