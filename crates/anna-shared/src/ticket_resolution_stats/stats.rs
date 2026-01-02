//! Ticket resolution statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{ResolutionMethod, ResolutionRecord, Resolver};

/// Ticket resolution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketResolutionStats {
    /// All resolution records
    pub records: Vec<ResolutionRecord>,
    /// Count by resolver
    pub by_resolver: HashMap<String, u64>,
    /// Count by method
    pub by_method: HashMap<String, u64>,
    /// Count by department
    pub by_department: HashMap<String, u64>,
    /// Total Anna resolutions
    pub anna_count: u64,
    /// Total specialist resolutions
    pub specialist_count: u64,
    /// Recipes learned
    pub recipes_learned: u64,
}

impl TicketResolutionStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a resolution
    pub fn record(&mut self, resolution: ResolutionRecord) {
        *self.by_resolver.entry(resolution.resolver.name().to_string()).or_insert(0) += 1;
        *self.by_method.entry(resolution.method.name().to_string()).or_insert(0) += 1;

        if let Some(dept) = &resolution.department {
            *self.by_department.entry(dept.clone()).or_insert(0) += 1;
        }

        if resolution.resolver == Resolver::Anna {
            self.anna_count += 1;
        } else if resolution.resolver.is_specialist() {
            self.specialist_count += 1;
        }

        if resolution.recipe_learned {
            self.recipes_learned += 1;
        }

        self.records.push(resolution);
    }

    /// Get Anna's resolution rate
    pub fn anna_rate(&self) -> f64 {
        let total = self.anna_count + self.specialist_count;
        if total == 0 {
            0.0
        } else {
            (self.anna_count as f64 / total as f64) * 100.0
        }
    }

    /// Get resolutions by resolver
    pub fn by_res(&self, resolver: Resolver) -> Vec<&ResolutionRecord> {
        self.records.iter().filter(|r| r.resolver == resolver).collect()
    }

    /// Get resolutions by method
    pub fn by_res_method(&self, method: ResolutionMethod) -> Vec<&ResolutionRecord> {
        self.records.iter().filter(|r| r.method == method).collect()
    }

    /// Get recent resolutions
    pub fn recent(&self, limit: usize) -> Vec<&ResolutionRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get recipe resolutions
    pub fn recipe_resolutions(&self) -> Vec<&ResolutionRecord> {
        self.records.iter().filter(|r| r.method == ResolutionMethod::Recipe).collect()
    }

    /// Average resolution time (seconds)
    pub fn avg_resolution_time(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let total: u64 = self.records.iter().map(|r| r.resolution_time_secs).sum();
        total as f64 / self.records.len() as f64
    }

    /// Fastest resolution
    pub fn fastest_resolution(&self) -> Option<u64> {
        self.records.iter().map(|r| r.resolution_time_secs).min()
    }

    /// Slowest resolution
    pub fn slowest_resolution(&self) -> Option<u64> {
        self.records.iter().map(|r| r.resolution_time_secs).max()
    }

    /// Most active department
    pub fn most_active_department(&self) -> Option<(&str, u64)> {
        self.by_department
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Total count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Anna is improving (higher rate over time)
    pub fn anna_improving(&self) -> bool {
        if self.records.len() < 20 {
            return false;
        }

        let mid = self.records.len() / 2;
        let first_half: Vec<_> = self.records[..mid].iter().collect();
        let second_half: Vec<_> = self.records[mid..].iter().collect();

        let first_anna = first_half.iter().filter(|r| r.resolver == Resolver::Anna).count();
        let second_anna = second_half.iter().filter(|r| r.resolver == Resolver::Anna).count();

        let first_rate = first_anna as f64 / first_half.len() as f64;
        let second_rate = second_anna as f64 / second_half.len() as f64;

        second_rate > first_rate
    }
}
