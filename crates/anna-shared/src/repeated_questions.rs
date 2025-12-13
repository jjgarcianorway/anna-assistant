//! Repeated Questions Detection (v0.0.485).
//!
//! Tracks and detects repeated or similar questions from users.
//! Helps identify patterns and opportunities for recipe learning.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Similarity threshold for considering questions as "same"
pub const SIMILARITY_THRESHOLD: f32 = 0.75;

/// Minimum times a question must appear to be "repeated"
pub const MIN_REPEAT_COUNT: u32 = 2;

/// A recorded question with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedQuestion {
    /// Normalized form of the question
    pub normalized: String,
    /// Original forms seen
    pub variants: Vec<String>,
    /// Number of times asked
    pub count: u32,
    /// Unix timestamp of first occurrence
    pub first_seen: u64,
    /// Unix timestamp of last occurrence
    pub last_seen: u64,
    /// Whether this was answered successfully
    pub resolved: bool,
    /// Category if detected
    pub category: Option<String>,
}

impl RecordedQuestion {
    /// Create a new recorded question
    pub fn new(question: &str, timestamp: u64) -> Self {
        Self {
            normalized: normalize_question(question),
            variants: vec![question.to_string()],
            count: 1,
            first_seen: timestamp,
            last_seen: timestamp,
            resolved: false,
            category: detect_category(question),
        }
    }

    /// Record another occurrence of this question
    pub fn record_occurrence(&mut self, question: &str, timestamp: u64) {
        self.count += 1;
        self.last_seen = timestamp;
        if !self.variants.contains(&question.to_string()) {
            self.variants.push(question.to_string());
        }
    }

    /// Mark as resolved
    pub fn mark_resolved(&mut self) {
        self.resolved = true;
    }

    /// Check if this is a repeated question
    pub fn is_repeated(&self) -> bool {
        self.count >= MIN_REPEAT_COUNT
    }

    /// Get days since first seen
    pub fn days_since_first(&self, now: u64) -> u64 {
        (now - self.first_seen) / 86400
    }
}

/// Question history tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestionHistory {
    /// Recorded questions indexed by normalized form
    pub questions: HashMap<String, RecordedQuestion>,
    /// Total questions recorded
    pub total_recorded: u64,
}

impl QuestionHistory {
    /// Create new empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a question
    pub fn record(&mut self, question: &str, timestamp: u64) -> Option<&RecordedQuestion> {
        self.total_recorded += 1;
        let normalized = normalize_question(question);

        // Check for exact normalized match
        if let Some(existing) = self.questions.get_mut(&normalized) {
            existing.record_occurrence(question, timestamp);
            return self.questions.get(&normalized);
        }

        // Check for similar questions
        if let Some(similar_key) = self.find_similar(&normalized) {
            if let Some(existing) = self.questions.get_mut(&similar_key) {
                existing.record_occurrence(question, timestamp);
                return self.questions.get(&similar_key);
            }
        }

        // New question
        self.questions
            .insert(normalized.clone(), RecordedQuestion::new(question, timestamp));
        self.questions.get(&normalized)
    }

    /// Find a similar existing question
    fn find_similar(&self, normalized: &str) -> Option<String> {
        for (key, _) in &self.questions {
            if calculate_similarity(key, normalized) >= SIMILARITY_THRESHOLD {
                return Some(key.clone());
            }
        }
        None
    }

    /// Get all repeated questions
    pub fn get_repeated(&self) -> Vec<&RecordedQuestion> {
        self.questions
            .values()
            .filter(|q| q.is_repeated())
            .collect()
    }

    /// Get top repeated questions by count
    pub fn top_repeated(&self, limit: usize) -> Vec<&RecordedQuestion> {
        let mut repeated: Vec<_> = self.get_repeated();
        repeated.sort_by(|a, b| b.count.cmp(&a.count));
        repeated.into_iter().take(limit).collect()
    }

    /// Get questions by category
    pub fn by_category(&self, category: &str) -> Vec<&RecordedQuestion> {
        self.questions
            .values()
            .filter(|q| q.category.as_deref() == Some(category))
            .collect()
    }

    /// Get unresolved repeated questions
    pub fn unresolved_repeated(&self) -> Vec<&RecordedQuestion> {
        self.questions
            .values()
            .filter(|q| q.is_repeated() && !q.resolved)
            .collect()
    }

    /// Count of repeated questions
    pub fn repeated_count(&self) -> usize {
        self.get_repeated().len()
    }

    /// Mark a question as resolved
    pub fn mark_resolved(&mut self, question: &str) {
        let normalized = normalize_question(question);
        if let Some(q) = self.questions.get_mut(&normalized) {
            q.mark_resolved();
        }
    }
}

/// Normalize a question for comparison
pub fn normalize_question(question: &str) -> String {
    let lower = question.to_lowercase();

    // Remove common filler phrases (must be whole words/phrases)
    let fillers = [
        "please ",
        "can you ",
        "could you ",
        "would you ",
        "i want to ",
        "i need to ",
        "i'd like to ",
        "help me ",
        "show me ",
        "tell me ",
        "how do i ",
        "how can i ",
        "what is ",
        "what's ",
    ];

    let mut result = format!(" {} ", lower);
    for filler in fillers {
        result = result.replace(filler, " ");
    }

    // Collapse whitespace and trim
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Calculate similarity between two normalized questions (0.0 to 1.0)
pub fn calculate_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    let words_a: Vec<&str> = a.split_whitespace().collect();
    let words_b: Vec<&str> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    // Count common words
    let common = words_a.iter().filter(|w| words_b.contains(w)).count();

    // Jaccard-like similarity
    let union = words_a.len() + words_b.len() - common;
    if union == 0 {
        return 0.0;
    }

    common as f32 / union as f32
}

/// Detect category from question text
pub fn detect_category(question: &str) -> Option<String> {
    let lower = question.to_lowercase();

    // Order matters: more specific categories first
    let categories = [
        ("docker", &["docker", "container", "kubernetes", "k8s"][..]),
        ("editor", &["vim", "nano", "emacs", "editor", "vimrc"]),
        ("git", &["git", "commit", "push", "pull", "branch"]),
        ("ssh", &["ssh", "sshd", "authorized_keys"]),
        ("package", &["install", "update", "upgrade", "package", "pacman", "yay"]),
        ("service", &["service", "systemd", "systemctl"]),
        ("network", &["network", "wifi", "ethernet", "ip", "dns", "connection"]),
        ("storage", &["disk", "storage", "space", "mount", "partition"]),
        ("system", &["cpu", "memory", "ram", "process", "load"]),
    ];

    for (category, keywords) in categories {
        for keyword in keywords {
            if lower.contains(keyword) {
                return Some(category.to_string());
            }
        }
    }

    None
}

/// Summary of repeated questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedQuestionsSummary {
    /// Total unique questions
    pub total_unique: usize,
    /// Number of repeated questions
    pub repeated_count: usize,
    /// Most repeated question
    pub most_repeated: Option<String>,
    /// Most repeated count
    pub most_repeated_count: u32,
    /// Categories with repeats
    pub categories_with_repeats: Vec<String>,
    /// Unresolved repeated count
    pub unresolved_count: usize,
}

impl QuestionHistory {
    /// Generate summary
    pub fn summary(&self) -> RepeatedQuestionsSummary {
        let repeated = self.get_repeated();
        let most_repeated = repeated.iter().max_by_key(|q| q.count);

        let mut categories: Vec<String> = repeated
            .iter()
            .filter_map(|q| q.category.clone())
            .collect();
        categories.sort();
        categories.dedup();

        RepeatedQuestionsSummary {
            total_unique: self.questions.len(),
            repeated_count: repeated.len(),
            most_repeated: most_repeated.map(|q| q.variants.first().cloned().unwrap_or_default()),
            most_repeated_count: most_repeated.map(|q| q.count).unwrap_or(0),
            categories_with_repeats: categories,
            unresolved_count: self.unresolved_repeated().len(),
        }
    }
}

/// Format repeated questions for display
pub fn format_repeated_questions(history: &QuestionHistory) -> String {
    let mut output = String::new();

    output.push_str("Repeated Questions\n");
    output.push_str("══════════════════════════════════════\n\n");

    let repeated = history.top_repeated(10);

    if repeated.is_empty() {
        output.push_str("No repeated questions detected yet.\n");
        return output;
    }

    for (i, q) in repeated.iter().enumerate() {
        let status = if q.resolved { "[OK]" } else { "[?]" };
        let category = q.category.as_deref().unwrap_or("general");

        output.push_str(&format!(
            "{}. {} ({}) - {} times\n",
            i + 1,
            q.variants.first().unwrap_or(&q.normalized),
            category,
            q.count
        ));
        output.push_str(&format!("   {} Status: {}\n", status, if q.resolved { "Resolved" } else { "Pending" }));

        if q.variants.len() > 1 {
            output.push_str("   Variants:\n");
            for variant in q.variants.iter().skip(1).take(3) {
                output.push_str(&format!("   - {}\n", variant));
            }
        }
        output.push('\n');
    }

    let summary = history.summary();
    output.push_str(&format!(
        "Summary: {} unique questions, {} repeated ({} unresolved)\n",
        summary.total_unique, summary.repeated_count, summary.unresolved_count
    ));

    output
}

/// Format compact repeated questions
pub fn format_repeated_compact(history: &QuestionHistory) -> String {
    let repeated = history.top_repeated(5);

    if repeated.is_empty() {
        return "No repeated questions".to_string();
    }

    let items: Vec<String> = repeated
        .iter()
        .map(|q| {
            let short = q
                .variants
                .first()
                .map(|s| if s.len() > 30 { format!("{}...", &s[..27]) } else { s.clone() })
                .unwrap_or_default();
            format!("\"{}\" ({}x)", short, q.count)
        })
        .collect();

    items.join(", ")
}

/// Check if query is asking about repeated questions
pub fn is_repeated_questions_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "repeated questions",
        "repeat questions",
        "common questions",
        "frequent questions",
        "asked questions",
        "what do i ask",
        "same questions",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_question() {
        assert_eq!(normalize_question("How do I install vim?"), "install vim?");
        assert_eq!(
            normalize_question("Please help me restart nginx"),
            "restart nginx"
        );
        assert_eq!(
            normalize_question("Can you show me disk usage"),
            "disk usage"
        );
    }

    #[test]
    fn test_calculate_similarity() {
        assert_eq!(calculate_similarity("install vim", "install vim"), 1.0);
        assert!(calculate_similarity("install vim", "install nano") > 0.3);
        assert!(calculate_similarity("install vim", "restart nginx") < 0.3);
    }

    #[test]
    fn test_detect_category() {
        assert_eq!(detect_category("install htop"), Some("package".to_string()));
        assert_eq!(
            detect_category("restart docker"),
            Some("docker".to_string())
        );
        assert_eq!(detect_category("vim config"), Some("editor".to_string()));
        assert_eq!(detect_category("random stuff"), None);
    }

    #[test]
    fn test_record_question() {
        let mut history = QuestionHistory::new();

        history.record("How do I install vim?", 1000);
        history.record("How do I install vim?", 2000);
        history.record("how can i install vim", 3000);

        // All three should be grouped (same normalized or similar)
        assert!(history.questions.len() <= 2);

        let repeated = history.get_repeated();
        assert!(!repeated.is_empty());
        // At least 2 occurrences of similar questions
        assert!(repeated.iter().any(|q| q.count >= 2));
    }

    #[test]
    fn test_similar_questions_grouped() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("please install vim", 2000);
        history.record("can you install vim", 3000);

        // Should be grouped as similar
        assert!(history.questions.len() <= 2);
    }

    #[test]
    fn test_top_repeated() {
        let mut history = QuestionHistory::new();

        // Question asked 5 times
        for i in 0..5 {
            history.record("install vim", 1000 + i * 100);
        }

        // Question asked 3 times
        for i in 0..3 {
            history.record("restart nginx", 2000 + i * 100);
        }

        // Question asked once
        history.record("disk usage", 3000);

        let top = history.top_repeated(10);
        assert!(top.len() >= 2);
        assert!(top[0].count >= top[1].count);
    }

    #[test]
    fn test_by_category() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install htop", 1100);
        history.record("restart docker", 2000);

        let packages = history.by_category("package");
        assert!(packages.len() >= 1);
    }

    #[test]
    fn test_mark_resolved() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install vim", 2000);

        history.mark_resolved("install vim");

        let unresolved = history.unresolved_repeated();
        assert!(unresolved.is_empty());
    }

    #[test]
    fn test_summary() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install vim", 2000);
        history.record("restart nginx", 3000);

        let summary = history.summary();
        assert_eq!(summary.total_unique, 2);
        assert_eq!(summary.repeated_count, 1);
    }

    #[test]
    fn test_format_repeated_compact() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install vim", 2000);
        history.record("install vim", 3000);

        let output = format_repeated_compact(&history);
        assert!(output.contains("3x"));
    }

    #[test]
    fn test_is_repeated_questions_query() {
        assert!(is_repeated_questions_query("show repeated questions"));
        assert!(is_repeated_questions_query("what are my common questions"));
        assert!(is_repeated_questions_query("frequent questions"));

        assert!(!is_repeated_questions_query("install vim"));
        assert!(!is_repeated_questions_query("status"));
    }

    #[test]
    fn test_recorded_question_days() {
        let q = RecordedQuestion::new("test", 0);
        assert_eq!(q.days_since_first(86400), 1);
        assert_eq!(q.days_since_first(86400 * 7), 7);
    }
}
