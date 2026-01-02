//! Query Pattern Analyzer Implementation
//!
//! Core analyzer for tracking and analyzing query patterns.

use super::types::{ConfidenceLevel, PatternCategory, QueryPattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query pattern analyzer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryPatternAnalyzer {
    /// All patterns
    pub patterns: Vec<QueryPattern>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Total matches
    pub total_matches: u64,
    /// Successful matches
    pub successful_matches: u64,
}

impl QueryPatternAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pattern
    pub fn add_pattern(&mut self, pattern: QueryPattern) {
        *self.by_category.entry(pattern.category.name().to_string()).or_insert(0) += 1;
        self.patterns.push(pattern);
    }

    /// Record a match
    pub fn record_match(&mut self, template: &str, success: bool, timestamp: u64) -> bool {
        let found = self.patterns.iter().position(|p| p.template == template);
        if let Some(idx) = found {
            self.patterns[idx].match_count += 1;
            self.patterns[idx].last_match = timestamp;
            self.total_matches += 1;
            if success {
                self.successful_matches += 1;
                // Update success rate
                let total = self.patterns[idx].match_count;
                let current_rate = self.patterns[idx].success_rate as u64;
                let new_rate = ((current_rate * (total - 1)) + 100) / total;
                self.patterns[idx].success_rate = new_rate as u8;
            }
            true
        } else {
            false
        }
    }

    /// Add example to pattern
    pub fn add_example(&mut self, template: &str, example: &str) {
        let found = self.patterns.iter().position(|p| p.template == template);
        if let Some(idx) = found {
            if self.patterns[idx].examples.len() < 5 {
                self.patterns[idx].examples.push(example.to_string());
            }
        }
    }

    /// Get pattern by template
    pub fn get(&self, template: &str) -> Option<&QueryPattern> {
        self.patterns.iter().find(|p| p.template == template)
    }

    /// Get patterns by category
    pub fn by_pat_category(&self, category: PatternCategory) -> Vec<&QueryPattern> {
        self.patterns.iter().filter(|p| p.category == category).collect()
    }

    /// Get high-confidence patterns
    pub fn high_confidence(&self) -> Vec<&QueryPattern> {
        self.patterns
            .iter()
            .filter(|p| p.confidence == ConfidenceLevel::High || p.confidence == ConfidenceLevel::Certain)
            .collect()
    }

    /// Get most used patterns
    pub fn most_used(&self, limit: usize) -> Vec<&QueryPattern> {
        let mut sorted: Vec<&QueryPattern> = self.patterns.iter().collect();
        sorted.sort_by(|a, b| b.match_count.cmp(&a.match_count));
        sorted.into_iter().take(limit).collect()
    }

    /// Get overall success rate
    pub fn overall_success_rate(&self) -> u8 {
        if self.total_matches == 0 {
            0
        } else {
            ((self.successful_matches * 100) / self.total_matches) as u8
        }
    }

    /// Total pattern count
    pub fn total_count(&self) -> usize {
        self.patterns.len()
    }
}
