// v0.0.601: Settings Comparator Implementation (Phase 177)
// Main comparator struct and methods

use super::types::CompareResult;

/// Settings comparator
#[derive(Debug, Clone, Default)]
pub struct SettingsComparator {
    /// Comparison history
    history: Vec<CompareResult>,
    /// Max history
    max_history: usize,
}

impl SettingsComparator {
    /// Create new comparator
    pub fn new() -> Self {
        Self {
            max_history: 50,
            ..Default::default()
        }
    }

    /// Record comparison
    pub fn record(&mut self, result: CompareResult) {
        self.history.push(result);
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get history
    pub fn history(&self) -> &[CompareResult] {
        &self.history
    }

    /// Recent comparisons
    pub fn recent(&self, count: usize) -> Vec<&CompareResult> {
        self.history.iter().rev().take(count).collect()
    }

    /// History count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparator_new() {
        let c = SettingsComparator::new();
        assert_eq!(c.history_count(), 0);
    }

    #[test]
    fn test_comparator_record() {
        let mut c = SettingsComparator::new();
        c.record(CompareResult::new("a", "b"));
        assert_eq!(c.history_count(), 1);
    }
}
