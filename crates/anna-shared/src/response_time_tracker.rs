// v0.0.538: Response Time Tracker (Phase 114)
// Tracks response times for "shortest/longest reply" per VISION.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ResponseType {
    #[default]
    Direct,
    Recipe,
    Specialist,
    Escalated,
    Research,
    Clarification,
}

impl std::fmt::Display for ResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "Direct"),
            Self::Recipe => write!(f, "Recipe"),
            Self::Specialist => write!(f, "Specialist"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Research => write!(f, "Research"),
            Self::Clarification => write!(f, "Clarification"),
        }
    }
}

/// Complexity level of the response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ComplexityLevel {
    #[default]
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "Simple"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Complex => write!(f, "Complex"),
            Self::VeryComplex => write!(f, "Very Complex"),
        }
    }
}

/// Single response time record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeRecord {
    pub id: String,
    pub ticket_id: Option<String>,
    pub response_type: ResponseType,
    pub complexity: ComplexityLevel,
    pub time_ms: u64,
    pub token_count: Option<u32>,
    pub word_count: u32,
    pub timestamp: DateTime<Utc>,
}

impl ResponseTimeRecord {
    /// Create new record
    pub fn new(id: impl Into<String>, time_ms: u64, word_count: u32) -> Self {
        Self {
            id: id.into(),
            ticket_id: None,
            response_type: ResponseType::default(),
            complexity: ComplexityLevel::default(),
            time_ms,
            token_count: None,
            word_count,
            timestamp: Utc::now(),
        }
    }

    /// Set response type
    pub fn with_type(mut self, response_type: ResponseType) -> Self {
        self.response_type = response_type;
        self
    }

    /// Set complexity
    pub fn with_complexity(mut self, complexity: ComplexityLevel) -> Self {
        self.complexity = complexity;
        self
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: impl Into<String>) -> Self {
        self.ticket_id = Some(ticket_id.into());
        self
    }

    /// Set token count
    pub fn with_tokens(mut self, count: u32) -> Self {
        self.token_count = Some(count);
        self
    }

    /// Words per second
    pub fn words_per_second(&self) -> f64 {
        if self.time_ms == 0 {
            return 0.0;
        }
        (self.word_count as f64 / self.time_ms as f64) * 1000.0
    }
}

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

/// Time distribution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeDistribution {
    pub min: u64,
    pub max: u64,
    pub avg: u64,
    pub std_dev: u64,
}

/// Format response time record
pub fn format_response_time(record: &ResponseTimeRecord) -> String {
    let mut output = format!(
        "Response {} [{}]\n  Time: {}ms | Words: {}\n  Type: {} | Complexity: {}\n",
        record.id, record.timestamp.format("%Y-%m-%d %H:%M"),
        record.time_ms, record.word_count,
        record.response_type, record.complexity
    );

    if let Some(tokens) = record.token_count {
        output.push_str(&format!("  Tokens: {}\n", tokens));
    }

    output.push_str(&format!("  Speed: {:.1} words/sec\n", record.words_per_second()));
    output
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &ResponseTimeTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Response Time Stats ===\n\n");

    output.push_str(&format!("Total Responses: {}\n", tracker.total()));

    if let Some(avg) = tracker.average_time_ms() {
        output.push_str(&format!("Average Time: {}ms\n", avg));
    }

    if let Some(avg) = tracker.average_word_count() {
        output.push_str(&format!("Average Words: {}\n", avg));
    }

    if let Some(shortest) = tracker.shortest_reply() {
        output.push_str(&format!("Shortest Reply: {} words ({})\n",
            shortest.word_count, shortest.id));
    }

    if let Some(longest) = tracker.longest_reply() {
        output.push_str(&format!("Longest Reply: {} words ({})\n",
            longest.word_count, longest.id));
    }

    if let Some(p95) = tracker.percentile_time(95) {
        output.push_str(&format!("95th Percentile: {}ms\n", p95));
    }

    output
}

/// Check if query is about response times
pub fn is_response_time_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("response time")
        || lower.contains("how long")
        || lower.contains("fastest")
        || lower.contains("slowest")
        || lower.contains("shortest reply")
        || lower.contains("longest reply")
}

/// Fun fact about response times
pub fn response_time_fun_fact() -> &'static str {
    "Anna tracks every response time! The 'shortest reply' might be a quick yes/no, while the 'longest reply' could be a full troubleshooting guide."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_type_default() {
        let rt = ResponseType::default();
        assert_eq!(rt, ResponseType::Direct);
    }

    #[test]
    fn test_complexity_default() {
        let c = ComplexityLevel::default();
        assert_eq!(c, ComplexityLevel::Simple);
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = ResponseTimeTracker::new();
        assert_eq!(tracker.total(), 0);
    }

    #[test]
    fn test_record_response() {
        let mut tracker = ResponseTimeTracker::new();
        let id = tracker.record(150, 25);
        assert!(tracker.get(&id).is_some());
        assert_eq!(tracker.total(), 1);
    }

    #[test]
    fn test_shortest_longest() {
        let mut tracker = ResponseTimeTracker::new();
        tracker.record(100, 10);
        tracker.record(500, 100);
        tracker.record(200, 50);

        let shortest = tracker.shortest_reply().unwrap();
        assert_eq!(shortest.word_count, 10);

        let longest = tracker.longest_reply().unwrap();
        assert_eq!(longest.word_count, 100);
    }

    #[test]
    fn test_average_time() {
        let mut tracker = ResponseTimeTracker::new();
        tracker.record(100, 10);
        tracker.record(200, 20);
        tracker.record(300, 30);

        let avg = tracker.average_time_ms().unwrap();
        assert_eq!(avg, 200);
    }

    #[test]
    fn test_type_stats() {
        let mut tracker = ResponseTimeTracker::new();
        tracker.record_full(100, 10, ResponseType::Direct, ComplexityLevel::Simple);
        tracker.record_full(200, 20, ResponseType::Direct, ComplexityLevel::Simple);
        tracker.record_full(300, 30, ResponseType::Specialist, ComplexityLevel::Complex);

        let stats = tracker.type_stats();
        assert_eq!(*stats.get(&ResponseType::Direct).unwrap_or(&0), 2);
        assert_eq!(*stats.get(&ResponseType::Specialist).unwrap_or(&0), 1);
    }

    #[test]
    fn test_percentile() {
        let mut tracker = ResponseTimeTracker::new();
        for i in 1..=100 {
            tracker.record(i * 10, 10);
        }

        let p95 = tracker.percentile_time(95).unwrap();
        assert!(p95 >= 900 && p95 <= 1000);
    }

    #[test]
    fn test_is_response_time_query() {
        assert!(is_response_time_query("What's my average response time?"));
        assert!(is_response_time_query("Show me the longest reply"));
        assert!(!is_response_time_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = response_time_fun_fact();
        assert!(fact.contains("response") || fact.contains("reply"));
    }

    #[test]
    fn test_words_per_second() {
        let record = ResponseTimeRecord::new("test", 1000, 50);
        assert!((record.words_per_second() - 50.0).abs() < 0.01);
    }
}
