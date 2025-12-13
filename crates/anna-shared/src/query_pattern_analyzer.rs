//! Query Pattern Analyzer - Phase 95
//!
//! Analyzes query patterns to improve Anna's understanding.
//! Learns from common query structures for better matching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query pattern category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PatternCategory {
    #[default]
    Question,
    Command,
    Status,
    Config,
    Troubleshoot,
    Install,
    Other,
}

impl PatternCategory {
    pub fn name(&self) -> &'static str {
        match self {
            PatternCategory::Question => "Question",
            PatternCategory::Command => "Command",
            PatternCategory::Status => "Status",
            PatternCategory::Config => "Config",
            PatternCategory::Troubleshoot => "Troubleshoot",
            PatternCategory::Install => "Install",
            PatternCategory::Other => "Other",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            PatternCategory::Question => "?",
            PatternCategory::Command => "!",
            PatternCategory::Status => "○",
            PatternCategory::Config => "⚙",
            PatternCategory::Troubleshoot => "⚡",
            PatternCategory::Install => "+",
            PatternCategory::Other => "·",
        }
    }
}

/// Pattern confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConfidenceLevel {
    #[default]
    Low,
    Medium,
    High,
    Certain,
}

impl ConfidenceLevel {
    pub fn name(&self) -> &'static str {
        match self {
            ConfidenceLevel::Low => "Low",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::High => "High",
            ConfidenceLevel::Certain => "Certain",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            ConfidenceLevel::Low => 25,
            ConfidenceLevel::Medium => 50,
            ConfidenceLevel::High => 75,
            ConfidenceLevel::Certain => 100,
        }
    }
}

/// A query pattern record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPattern {
    /// Pattern template (e.g., "how to {action} {target}")
    pub template: String,
    /// Category
    pub category: PatternCategory,
    /// Confidence level
    pub confidence: ConfidenceLevel,
    /// Number of matches
    pub match_count: u64,
    /// Success rate (0-100)
    pub success_rate: u8,
    /// Example queries that matched
    pub examples: Vec<String>,
    /// Last matched timestamp
    pub last_match: u64,
}

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

/// Format pattern analyzer for display
pub fn format_pattern_analyzer(analyzer: &QueryPatternAnalyzer) -> String {
    let mut lines = vec!["=== Query Pattern Analyzer ===".to_string()];
    lines.push(String::new());

    if analyzer.patterns.is_empty() {
        lines.push("No patterns analyzed yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total patterns: {}", analyzer.total_count()));
    lines.push(format!("Total matches: {}", analyzer.total_matches));
    lines.push(format!("Success rate: {}%", analyzer.overall_success_rate()));

    // By category
    if !analyzer.by_category.is_empty() {
        lines.push(String::new());
        lines.push("By category:".to_string());
        for (cat, count) in &analyzer.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    // Most used
    let most_used = analyzer.most_used(5);
    if !most_used.is_empty() {
        lines.push(String::new());
        lines.push("Most used patterns:".to_string());
        for pattern in most_used {
            lines.push(format!(
                "  [{}] {} ({} matches, {}% success)",
                pattern.category.symbol(),
                pattern.template,
                pattern.match_count,
                pattern.success_rate
            ));
        }
    }

    lines.join("\n")
}

/// Format pattern analyzer compact
pub fn format_pattern_analyzer_compact(analyzer: &QueryPatternAnalyzer) -> String {
    format!(
        "Patterns: {} analyzed | {} matches | {}% success",
        analyzer.total_count(),
        analyzer.total_matches,
        analyzer.overall_success_rate()
    )
}

/// Format pattern analyzer one-line
pub fn format_pattern_analyzer_oneline(analyzer: &QueryPatternAnalyzer) -> String {
    format!(
        "{} patterns ({}% success)",
        analyzer.total_count(),
        analyzer.overall_success_rate()
    )
}

/// Check if query is about patterns
pub fn is_pattern_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "pattern",
        "patterns",
        "query pattern",
        "how do i ask",
        "query analysis",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about patterns
pub fn pattern_fun_fact(analyzer: &QueryPatternAnalyzer) -> String {
    if analyzer.patterns.is_empty() {
        return "No patterns analyzed yet!".to_string();
    }

    let facts = [
        format!("Anna knows {} query patterns.", analyzer.total_count()),
        format!("Patterns have been matched {} times.", analyzer.total_matches),
        format!("Overall success rate is {}%.", analyzer.overall_success_rate()),
    ];

    facts[analyzer.total_count() % facts.len()].clone()
}

/// Common query patterns
pub const COMMON_PATTERNS: &[(&str, PatternCategory)] = &[
    ("how to {action} {target}", PatternCategory::Question),
    ("what is {target}", PatternCategory::Question),
    ("why is {target} {state}", PatternCategory::Troubleshoot),
    ("install {package}", PatternCategory::Install),
    ("remove {package}", PatternCategory::Command),
    ("start {service}", PatternCategory::Command),
    ("stop {service}", PatternCategory::Command),
    ("restart {service}", PatternCategory::Command),
    ("status of {target}", PatternCategory::Status),
    ("show {target}", PatternCategory::Status),
    ("configure {target}", PatternCategory::Config),
    ("set {setting} to {value}", PatternCategory::Config),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pattern(template: &str, category: PatternCategory) -> QueryPattern {
        QueryPattern {
            template: template.to_string(),
            category,
            confidence: ConfidenceLevel::Medium,
            match_count: 0,
            success_rate: 80,
            examples: vec![],
            last_match: 0,
        }
    }

    #[test]
    fn test_pattern_category() {
        assert_eq!(PatternCategory::Question.name(), "Question");
        assert_eq!(PatternCategory::Install.symbol(), "+");
    }

    #[test]
    fn test_confidence_level() {
        assert_eq!(ConfidenceLevel::High.name(), "High");
        assert_eq!(ConfidenceLevel::High.score(), 75);
    }

    #[test]
    fn test_add_pattern() {
        let mut analyzer = QueryPatternAnalyzer::new();
        analyzer.add_pattern(make_pattern("how to {action}", PatternCategory::Question));

        assert_eq!(analyzer.total_count(), 1);
        assert!(analyzer.get("how to {action}").is_some());
    }

    #[test]
    fn test_record_match() {
        let mut analyzer = QueryPatternAnalyzer::new();
        analyzer.add_pattern(make_pattern("how to {action}", PatternCategory::Question));
        analyzer.record_match("how to {action}", true, 1000);

        assert_eq!(analyzer.total_matches, 1);
        assert_eq!(analyzer.successful_matches, 1);
        assert_eq!(analyzer.get("how to {action}").unwrap().match_count, 1);
    }

    #[test]
    fn test_add_example() {
        let mut analyzer = QueryPatternAnalyzer::new();
        analyzer.add_pattern(make_pattern("how to {action}", PatternCategory::Question));
        analyzer.add_example("how to {action}", "how to install firefox");

        assert_eq!(analyzer.get("how to {action}").unwrap().examples.len(), 1);
    }

    #[test]
    fn test_by_category() {
        let mut analyzer = QueryPatternAnalyzer::new();
        analyzer.add_pattern(make_pattern("how to {action}", PatternCategory::Question));
        analyzer.add_pattern(make_pattern("install {pkg}", PatternCategory::Install));

        assert_eq!(analyzer.by_pat_category(PatternCategory::Question).len(), 1);
        assert_eq!(analyzer.by_pat_category(PatternCategory::Install).len(), 1);
    }

    #[test]
    fn test_most_used() {
        let mut analyzer = QueryPatternAnalyzer::new();
        analyzer.add_pattern(make_pattern("how to {action}", PatternCategory::Question));
        analyzer.add_pattern(make_pattern("install {pkg}", PatternCategory::Install));

        // Record more matches for install pattern
        analyzer.record_match("install {pkg}", true, 1000);
        analyzer.record_match("install {pkg}", true, 2000);

        let most_used = analyzer.most_used(1);
        assert_eq!(most_used[0].template, "install {pkg}");
    }

    #[test]
    fn test_success_rate() {
        let mut analyzer = QueryPatternAnalyzer::new();
        analyzer.add_pattern(make_pattern("test", PatternCategory::Other));
        analyzer.record_match("test", true, 1000);
        analyzer.record_match("test", false, 2000);

        assert_eq!(analyzer.overall_success_rate(), 50);
    }

    #[test]
    fn test_format_analyzer() {
        let mut analyzer = QueryPatternAnalyzer::new();
        analyzer.add_pattern(make_pattern("how to {action}", PatternCategory::Question));

        let output = format_pattern_analyzer(&analyzer);
        assert!(output.contains("Query Pattern Analyzer"));
        assert!(output.contains("Total patterns: 1"));
    }

    #[test]
    fn test_is_pattern_query() {
        assert!(is_pattern_query("show query patterns"));
        assert!(is_pattern_query("how do I ask about packages?"));
        assert!(!is_pattern_query("what is the weather?"));
    }

    #[test]
    fn test_common_patterns() {
        assert!(!COMMON_PATTERNS.is_empty());
        assert!(COMMON_PATTERNS.iter().any(|(t, _)| t.contains("install")));
    }
}
