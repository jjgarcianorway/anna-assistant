// v0.0.537: Query History Tracker Implementation (Phase 113)
// Main tracker logic and query management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{QueryCategory, QueryOutcome, QueryRecord};
use super::utils::{classify_query, query_similarity};

/// Query history tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryTracker {
    queries: HashMap<String, QueryRecord>,
    next_id: u64,
    similarity_threshold: f32,
}

impl Default for QueryHistoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryHistoryTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            queries: HashMap::new(),
            next_id: 1,
            similarity_threshold: 0.8,
        }
    }

    /// Record a new query
    pub fn record(&mut self, query_text: impl Into<String>) -> String {
        let id = format!("Q{:05}", self.next_id);
        self.next_id += 1;

        let text = query_text.into();
        let category = classify_query(&text);
        let mut record = QueryRecord::new(&id, text).with_category(category);

        // Check for similar queries
        record.similar_count = self.count_similar(&record.normalized_text);

        self.queries.insert(id.clone(), record);
        id
    }

    /// Get query by ID
    pub fn get(&self, id: &str) -> Option<&QueryRecord> {
        self.queries.get(id)
    }

    /// Get mutable query by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut QueryRecord> {
        self.queries.get_mut(id)
    }

    /// Mark query resolved
    pub fn resolve(&mut self, id: &str, response_time_ms: u64) {
        if let Some(q) = self.queries.get_mut(id) {
            q.outcome = QueryOutcome::Resolved;
            q.response_time_ms = Some(response_time_ms);
        }
    }

    /// Mark query escalated
    pub fn escalate(&mut self, id: &str) {
        if let Some(q) = self.queries.get_mut(id) {
            q.outcome = QueryOutcome::Escalated;
        }
    }

    /// Mark query failed
    pub fn fail(&mut self, id: &str) {
        if let Some(q) = self.queries.get_mut(id) {
            q.outcome = QueryOutcome::Failed;
        }
    }

    /// Count similar queries
    fn count_similar(&self, normalized: &str) -> u32 {
        self.queries
            .values()
            .filter(|q| query_similarity(&q.normalized_text, normalized) >= self.similarity_threshold)
            .count() as u32
    }

    /// Get repeated queries (asked more than once)
    pub fn repeated_queries(&self) -> Vec<&QueryRecord> {
        let mut counts: HashMap<String, Vec<&QueryRecord>> = HashMap::new();
        for q in self.queries.values() {
            counts.entry(q.normalized_text.clone()).or_default().push(q);
        }

        let mut repeated: Vec<&QueryRecord> = counts
            .into_iter()
            .filter(|(_, qs)| qs.len() > 1)
            .flat_map(|(_, qs)| qs)
            .collect();

        repeated.sort_by(|a, b| b.similar_count.cmp(&a.similar_count));
        repeated
    }

    /// Get category stats (topic most asked about)
    pub fn category_stats(&self) -> Vec<(QueryCategory, u32)> {
        let mut counts: HashMap<QueryCategory, u32> = HashMap::new();
        for q in self.queries.values() {
            *counts.entry(q.category.clone()).or_default() += 1;
        }

        let mut stats: Vec<_> = counts.into_iter().collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1));
        stats
    }

    /// Get most asked topic
    pub fn most_asked_topic(&self) -> Option<QueryCategory> {
        self.category_stats().into_iter().next().map(|(c, _)| c)
    }

    /// Get queries by category
    pub fn by_category(&self, category: &QueryCategory) -> Vec<&QueryRecord> {
        self.queries.values().filter(|q| &q.category == category).collect()
    }

    /// Get queries by outcome
    pub fn by_outcome(&self, outcome: QueryOutcome) -> Vec<&QueryRecord> {
        self.queries.values().filter(|q| q.outcome == outcome).collect()
    }

    /// Get recent queries
    pub fn recent(&self, limit: usize) -> Vec<&QueryRecord> {
        let mut queries: Vec<_> = self.queries.values().collect();
        queries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        queries.into_iter().take(limit).collect()
    }

    /// Total query count
    pub fn total(&self) -> usize {
        self.queries.len()
    }

    /// Resolution stats
    pub fn resolution_stats(&self) -> HashMap<QueryOutcome, u32> {
        let mut counts = HashMap::new();
        for q in self.queries.values() {
            *counts.entry(q.outcome).or_default() += 1;
        }
        counts
    }

    /// Average response time
    pub fn average_response_time_ms(&self) -> Option<u64> {
        let times: Vec<u64> = self.queries.values()
            .filter_map(|q| q.response_time_ms)
            .collect();
        if times.is_empty() {
            None
        } else {
            Some(times.iter().sum::<u64>() / times.len() as u64)
        }
    }
}
