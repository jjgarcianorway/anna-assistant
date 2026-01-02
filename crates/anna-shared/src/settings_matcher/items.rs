// v0.0.687: Settings Matcher Items (Phase 263)
// Match items and results

use serde::{Deserialize, Serialize};

use super::types::MatchTarget;

/// Match item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Matched rules
    pub matched_rules: Vec<String>,
    /// Match target
    pub matched_on: MatchTarget,
}

impl MatchItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, rules: Vec<String>, target: MatchTarget) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            matched_rules: rules,
            matched_on: target,
        }
    }

    /// How many rules matched
    pub fn rule_count(&self) -> usize {
        self.matched_rules.len()
    }
}

/// Match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Matched items
    pub items: Vec<MatchItem>,
    /// Total checked
    pub total_checked: usize,
    /// Total matched
    pub total_matched: usize,
    /// Rules applied
    pub rules_applied: usize,
}

impl MatchResult {
    /// Create new result
    pub fn new(items: Vec<MatchItem>, checked: usize, rules: usize) -> Self {
        let total_matched = items.len();
        Self {
            items,
            total_checked: checked,
            total_matched,
            rules_applied: rules,
        }
    }

    /// Has matches
    pub fn has_matches(&self) -> bool {
        !self.items.is_empty()
    }

    /// Match rate
    pub fn match_rate(&self) -> f64 {
        if self.total_checked == 0 {
            0.0
        } else {
            self.total_matched as f64 / self.total_checked as f64
        }
    }

    /// Filter by rule
    pub fn filter_by_rule(&self, rule_id: &str) -> Vec<&MatchItem> {
        self.items.iter().filter(|i| i.matched_rules.contains(&rule_id.to_string())).collect()
    }
}

impl Default for MatchResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_item_new() {
        let i = MatchItem::new("key", "value", vec!["r1".to_string()], MatchTarget::Key);
        assert_eq!(i.rule_count(), 1);
    }

    #[test]
    fn test_result_new() {
        let r = MatchResult::new(vec![MatchItem::new("k", "v", vec![], MatchTarget::Key)], 10, 2);
        assert!(r.has_matches());
    }

    #[test]
    fn test_result_match_rate() {
        let items = vec![MatchItem::new("k", "v", vec![], MatchTarget::Key)];
        let r = MatchResult::new(items, 10, 1);
        assert!((r.match_rate() - 0.1).abs() < 0.001);
    }
}
