// v0.0.654: Settings Injector Result (Phase 230)
// Result and statistics for settings injection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::InjectionType;

/// Injection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionResult {
    /// Keys inserted
    pub inserted: Vec<String>,
    /// Keys updated
    pub updated: Vec<String>,
    /// Keys skipped
    pub skipped: Vec<String>,
    /// Keys failed
    pub failed: Vec<String>,
    /// Injection type used
    pub injection_type: InjectionType,
}

impl InjectionResult {
    /// Create new result
    pub fn new(injection_type: InjectionType) -> Self {
        Self {
            inserted: Vec::new(),
            updated: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            injection_type,
        }
    }

    /// Add inserted
    pub fn add_inserted(&mut self, key: String) {
        self.inserted.push(key);
    }

    /// Add updated
    pub fn add_updated(&mut self, key: String) {
        self.updated.push(key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.skipped.push(key);
    }

    /// Add failed
    pub fn add_failed(&mut self, key: String) {
        self.failed.push(key);
    }

    /// Total affected
    pub fn total_affected(&self) -> usize {
        self.inserted.len() + self.updated.len()
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }
}

/// Injector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InjectorStats {
    /// Total injections
    pub total_injections: usize,
    /// Total inserted
    pub total_inserted: usize,
    /// Total updated
    pub total_updated: usize,
    /// Total skipped
    pub total_skipped: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl InjectorStats {
    /// Record injection
    pub fn record(&mut self, injection_type: InjectionType, inserted: usize, updated: usize, skipped: usize) {
        self.total_injections += 1;
        self.total_inserted += inserted;
        self.total_updated += updated;
        self.total_skipped += skipped;
        *self.by_type.entry(injection_type.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_inserted + self.total_updated + self.total_skipped;
        if total == 0 {
            0.0
        } else {
            (self.total_inserted + self.total_updated) as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_new() {
        let r = InjectionResult::new(InjectionType::Insert);
        assert_eq!(r.total_affected(), 0);
    }

    #[test]
    fn test_result_add() {
        let mut r = InjectionResult::new(InjectionType::Insert);
        r.add_inserted("key1".to_string());
        r.add_updated("key2".to_string());
        assert_eq!(r.total_affected(), 2);
    }

    #[test]
    fn test_stats_record() {
        let mut s = InjectorStats::default();
        s.record(InjectionType::Insert, 5, 3, 2);
        assert_eq!(s.total_injections, 1);
        assert_eq!(s.total_inserted, 5);
    }
}
