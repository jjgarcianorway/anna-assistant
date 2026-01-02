// v0.0.687: Settings Matcher Stats (Phase 263)
// Statistics tracking for matcher

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::items::MatchResult;
use super::types::MatchType;

/// Matcher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatcherStats {
    /// Total matches
    pub total_matches: usize,
    /// Total checked
    pub total_checked: usize,
    /// Total matched
    pub total_matched: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MatcherStats {
    /// Record match
    pub fn record(&mut self, result: &MatchResult, match_type: MatchType) {
        self.total_matches += 1;
        self.total_checked += result.total_checked;
        self.total_matched += result.total_matched;
        *self.by_type.entry(match_type.to_string()).or_insert(0) += 1;
    }

    /// Overall match rate
    pub fn overall_rate(&self) -> f64 {
        if self.total_checked == 0 {
            0.0
        } else {
            self.total_matched as f64 / self.total_checked as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_matcher::items::MatchItem;
    use crate::settings_matcher::types::MatchTarget;

    #[test]
    fn test_stats_record() {
        let mut s = MatcherStats::default();
        let r = MatchResult::new(vec![MatchItem::new("k", "v", vec![], MatchTarget::Key)], 10, 1);
        s.record(&r, MatchType::Contains);
        assert_eq!(s.total_matches, 1);
    }
}
