// v0.0.537: Query History Tracker (Phase 113)
// Tracks user queries for "repeated questions" and "topic most asked about" per VISION.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query category for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryCategory {
    System,
    Network,
    Storage,
    Audio,
    Video,
    Desktop,
    Security,
    Package,
    Service,
    Editor,
    Shell,
    Hardware,
    Configuration,
    General,
    Custom(String),
}

impl Default for QueryCategory {
    fn default() -> Self {
        Self::General
    }
}

impl std::fmt::Display for QueryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "System"),
            Self::Network => write!(f, "Network"),
            Self::Storage => write!(f, "Storage"),
            Self::Audio => write!(f, "Audio"),
            Self::Video => write!(f, "Video"),
            Self::Desktop => write!(f, "Desktop"),
            Self::Security => write!(f, "Security"),
            Self::Package => write!(f, "Package"),
            Self::Service => write!(f, "Service"),
            Self::Editor => write!(f, "Editor"),
            Self::Shell => write!(f, "Shell"),
            Self::Hardware => write!(f, "Hardware"),
            Self::Configuration => write!(f, "Configuration"),
            Self::General => write!(f, "General"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Query resolution outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum QueryOutcome {
    #[default]
    Pending,
    Resolved,
    Escalated,
    Failed,
    Deferred,
}

impl std::fmt::Display for QueryOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Failed => write!(f, "Failed"),
            Self::Deferred => write!(f, "Deferred"),
        }
    }
}

/// Single query record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRecord {
    pub id: String,
    pub query_text: String,
    pub normalized_text: String,
    pub category: QueryCategory,
    pub outcome: QueryOutcome,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub ticket_id: Option<String>,
    pub similar_count: u32,
}

impl QueryRecord {
    /// Create new query record
    pub fn new(id: impl Into<String>, query_text: impl Into<String>) -> Self {
        let text = query_text.into();
        Self {
            id: id.into(),
            normalized_text: normalize_query(&text),
            query_text: text,
            category: QueryCategory::default(),
            outcome: QueryOutcome::default(),
            timestamp: Utc::now(),
            response_time_ms: None,
            ticket_id: None,
            similar_count: 0,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: QueryCategory) -> Self {
        self.category = category;
        self
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: impl Into<String>) -> Self {
        self.ticket_id = Some(ticket_id.into());
        self
    }
}

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

/// Normalize query for comparison
fn normalize_query(query: &str) -> String {
    query.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Simple word-based similarity
fn query_similarity(a: &str, b: &str) -> f32 {
    let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count() as f32;
    let union = words_a.union(&words_b).count() as f32;

    intersection / union
}

/// Classify query into category
fn classify_query(query: &str) -> QueryCategory {
    let lower = query.to_lowercase();

    // Check specific tool names first (more specific matches)
    if lower.contains("vim") || lower.contains("nano") || lower.contains("neovim")
        || lower.contains("emacs") || lower.contains("vscode") || lower.contains("editor") {
        QueryCategory::Editor
    } else if lower.contains("network") || lower.contains("wifi") || lower.contains("ethernet")
        || lower.contains("ip address") || lower.contains("dns") || lower.contains("firewall") {
        QueryCategory::Network
    } else if lower.contains("disk") || lower.contains("storage") || lower.contains("mount")
        || lower.contains("partition") || lower.contains("filesystem") {
        QueryCategory::Storage
    } else if lower.contains("audio") || lower.contains("sound") || lower.contains("speaker")
        || lower.contains("microphone") || lower.contains("pulseaudio") || lower.contains("pipewire") {
        QueryCategory::Audio
    } else if lower.contains("video") || lower.contains("display") || lower.contains("monitor")
        || lower.contains("resolution") || lower.contains("gpu") || lower.contains("graphics") {
        QueryCategory::Video
    } else if lower.contains("desktop") || lower.contains("window manager") || lower.contains("theme")
        || lower.contains("kde") || lower.contains("gnome") || lower.contains("xfce") {
        QueryCategory::Desktop
    } else if lower.contains("security") || lower.contains("password") || lower.contains("permission")
        || lower.contains("sudo") || lower.contains("root access") || lower.contains("ssh") {
        QueryCategory::Security
    } else if lower.contains("install") || lower.contains("package") || lower.contains("pacman")
        || lower.contains("yay") || lower.contains("aur") || lower.contains("update package") {
        QueryCategory::Package
    } else if lower.contains("service") || lower.contains("systemd") || lower.contains("daemon")
        || lower.contains("systemctl") {
        QueryCategory::Service
    } else if lower.contains("bash") || lower.contains("zsh") || lower.contains("shell")
        || lower.contains("terminal") || lower.contains("command line") || lower.contains("script") {
        QueryCategory::Shell
    } else if lower.contains("cpu") || lower.contains("ram") || lower.contains("hardware")
        || lower.contains("memory") || lower.contains("temperature") || lower.contains("fan") {
        QueryCategory::Hardware
    } else if lower.contains("config") || lower.contains("setting") || lower.contains("option")
        || lower.contains("preference") || lower.contains("customize") {
        QueryCategory::Configuration
    } else if lower.contains("system") || lower.contains("boot") || lower.contains("kernel")
        || lower.contains("grub") || lower.contains("linux") || lower.contains("arch") {
        QueryCategory::System
    } else {
        QueryCategory::General
    }
}

/// Format query record
pub fn format_query(record: &QueryRecord) -> String {
    let mut output = format!(
        "Query {} [{}]\n  \"{}\"\n  Category: {} | Outcome: {}\n",
        record.id, record.timestamp.format("%Y-%m-%d %H:%M"),
        record.query_text, record.category, record.outcome
    );

    if let Some(time) = record.response_time_ms {
        output.push_str(&format!("  Response: {}ms\n", time));
    }

    if record.similar_count > 0 {
        output.push_str(&format!("  Similar queries: {}\n", record.similar_count));
    }

    output
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &QueryHistoryTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Query History ===\n\n");

    output.push_str(&format!("Total Queries: {}\n", tracker.total()));

    if let Some(avg) = tracker.average_response_time_ms() {
        output.push_str(&format!("Avg Response Time: {}ms\n", avg));
    }

    output.push_str("\nBy Category:\n");
    for (cat, count) in tracker.category_stats().iter().take(5) {
        output.push_str(&format!("  {}: {}\n", cat, count));
    }

    output.push_str("\nBy Outcome:\n");
    for (outcome, count) in tracker.resolution_stats() {
        output.push_str(&format!("  {}: {}\n", outcome, count));
    }

    let repeated = tracker.repeated_queries();
    if !repeated.is_empty() {
        output.push_str(&format!("\nRepeated Queries: {}\n", repeated.len()));
    }

    output
}

/// Check if query is history-related
pub fn is_history_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("history")
        || lower.contains("previous question")
        || lower.contains("asked before")
        || lower.contains("repeated")
        || lower.contains("most asked")
        || lower.contains("common question")
}

/// Fun fact about query history
pub fn query_history_fun_fact() -> &'static str {
    "Anna remembers every question you ask! The 'repeated questions' stat helps identify topics you might need a permanent solution for."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_category_default() {
        let cat = QueryCategory::default();
        assert_eq!(cat, QueryCategory::General);
    }

    #[test]
    fn test_query_outcome_default() {
        let outcome = QueryOutcome::default();
        assert_eq!(outcome, QueryOutcome::Pending);
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = QueryHistoryTracker::new();
        assert_eq!(tracker.total(), 0);
    }

    #[test]
    fn test_record_query() {
        let mut tracker = QueryHistoryTracker::new();
        let id = tracker.record("How do I install vim?");
        assert!(tracker.get(&id).is_some());
        assert_eq!(tracker.total(), 1);
    }

    #[test]
    fn test_resolve_query() {
        let mut tracker = QueryHistoryTracker::new();
        let id = tracker.record("Test query");
        tracker.resolve(&id, 150);

        let q = tracker.get(&id).unwrap();
        assert_eq!(q.outcome, QueryOutcome::Resolved);
        assert_eq!(q.response_time_ms, Some(150));
    }

    #[test]
    fn test_classify_network() {
        let cat = classify_query("How do I configure wifi?");
        assert_eq!(cat, QueryCategory::Network);
    }

    #[test]
    fn test_classify_editor() {
        let cat = classify_query("Enable syntax highlighting in vim");
        assert_eq!(cat, QueryCategory::Editor);
    }

    #[test]
    fn test_category_stats() {
        let mut tracker = QueryHistoryTracker::new();
        tracker.record("Install vim");
        tracker.record("Install nano");
        tracker.record("Configure wifi");

        let stats = tracker.category_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_normalize_query() {
        let normalized = normalize_query("How do I INSTALL vim??");
        assert_eq!(normalized, "how do i install vim");
    }

    #[test]
    fn test_is_history_query() {
        assert!(is_history_query("Show my query history"));
        assert!(is_history_query("What have I asked before?"));
        assert!(!is_history_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = query_history_fun_fact();
        assert!(fact.contains("repeated") || fact.contains("question"));
    }
}
