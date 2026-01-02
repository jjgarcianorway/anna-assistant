// v0.0.538: Response Time Tracker - Tracker (Phase 114)
// Main tracker for managing response time records

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::record::ResponseTimeRecord;
use super::types::{ComplexityLevel, ResponseType, TimeDistribution};

/// Response time tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeTracker {
    records: HashMap<String, ResponseTimeRecord>,
    next_id: u64,
}

impl Default for ResponseTimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseTimeTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            next_id: 1,
        }
    }

    /// Record response time
    pub fn record(&mut self, time_ms: u64, word_count: u32) -> String {
        let id = format!("RT{:05}", self.next_id);
        self.next_id += 1;

        let record = ResponseTimeRecord::new(&id, time_ms, word_count);
        self.records.insert(id.clone(), record);
        id
    }

    /// Record with full details
    pub fn record_full(
        &mut self,
        time_ms: u64,
        word_count: u32,
        response_type: ResponseType,
        complexity: ComplexityLevel,
    ) -> String {
        let id = format!("RT{:05}", self.next_id);
        self.next_id += 1;

        let record = ResponseTimeRecord::new(&id, time_ms, word_count)
            .with_type(response_type)
            .with_complexity(complexity);

        self.records.insert(id.clone(), record);
        id
    }

    /// Get record by ID
    pub fn get(&self, id: &str) -> Option<&ResponseTimeRecord> {
        self.records.get(id)
    }

    /// Get mutable record by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ResponseTimeRecord> {
        self.records.get_mut(id)
    }

    /// Get shortest response (by time)
    pub fn shortest_time(&self) -> Option<&ResponseTimeRecord> {
        self.records.values().min_by_key(|r| r.time_ms)
    }

    /// Get longest response (by time)
    pub fn longest_time(&self) -> Option<&ResponseTimeRecord> {
        self.records.values().max_by_key(|r| r.time_ms)
    }

    /// Get shortest reply (by word count)
    pub fn shortest_reply(&self) -> Option<&ResponseTimeRecord> {
        self.records.values().min_by_key(|r| r.word_count)
    }

    /// Get longest reply (by word count)
    pub fn longest_reply(&self) -> Option<&ResponseTimeRecord> {
        self.records.values().max_by_key(|r| r.word_count)
    }

    /// Average response time
    pub fn average_time_ms(&self) -> Option<u64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: u64 = self.records.values().map(|r| r.time_ms).sum();
        Some(sum / self.records.len() as u64)
    }

    /// Average word count
    pub fn average_word_count(&self) -> Option<u32> {
        if self.records.is_empty() {
            return None;
        }
        let sum: u32 = self.records.values().map(|r| r.word_count).sum();
        Some(sum / self.records.len() as u32)
    }

    /// Get by response type
    pub fn by_type(&self, response_type: ResponseType) -> Vec<&ResponseTimeRecord> {
        self.records
            .values()
            .filter(|r| r.response_type == response_type)
            .collect()
    }

    /// Get by complexity
    pub fn by_complexity(&self, complexity: ComplexityLevel) -> Vec<&ResponseTimeRecord> {
        self.records
            .values()
            .filter(|r| r.complexity == complexity)
            .collect()
    }

    /// Type stats
    pub fn type_stats(&self) -> HashMap<ResponseType, u32> {
        let mut counts = HashMap::new();
        for r in self.records.values() {
            *counts.entry(r.response_type).or_default() += 1;
        }
        counts
    }

    /// Complexity stats
    pub fn complexity_stats(&self) -> HashMap<ComplexityLevel, u32> {
        let mut counts = HashMap::new();
        for r in self.records.values() {
            *counts.entry(r.complexity).or_default() += 1;
        }
        counts
    }

    /// Recent records
    pub fn recent(&self, limit: usize) -> Vec<&ResponseTimeRecord> {
        let mut records: Vec<_> = self.records.values().collect();
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.into_iter().take(limit).collect()
    }

    /// Total count
    pub fn total(&self) -> usize {
        self.records.len()
    }

    /// Percentile time (e.g., 95th percentile)
    pub fn percentile_time(&self, percentile: u8) -> Option<u64> {
        if self.records.is_empty() {
            return None;
        }

        let mut times: Vec<u64> = self.records.values().map(|r| r.time_ms).collect();
        times.sort();

        let idx = ((percentile as f64 / 100.0) * times.len() as f64) as usize;
        let idx = idx.min(times.len() - 1);
        Some(times[idx])
    }

    /// Time distribution stats
    pub fn time_distribution(&self) -> TimeDistribution {
        let times: Vec<u64> = self.records.values().map(|r| r.time_ms).collect();
        if times.is_empty() {
            return TimeDistribution::default();
        }

        let min = *times.iter().min().unwrap();
        let max = *times.iter().max().unwrap();
        let sum: u64 = times.iter().sum();
        let avg = sum / times.len() as u64;

        // Standard deviation
        let variance = times.iter()
            .map(|t| (*t as f64 - avg as f64).powi(2))
            .sum::<f64>() / times.len() as f64;
        let std_dev = variance.sqrt() as u64;

        TimeDistribution { min, max, avg, std_dev }
    }
}
