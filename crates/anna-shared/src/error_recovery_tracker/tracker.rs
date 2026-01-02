//! Error recovery tracker implementation
//!
//! Tracks error recovery attempts and success rates.

use super::types::{ErrorCategory, ErrorRecoveryRecord, RecoveryOutcome};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
