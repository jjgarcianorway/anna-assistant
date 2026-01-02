//! Query Pattern Analyzer Tests

#![cfg(test)]

use super::analyzer::QueryPatternAnalyzer;
use super::formatters::format_pattern_analyzer;
use super::types::{ConfidenceLevel, PatternCategory, QueryPattern};
use super::utils::{is_pattern_query, COMMON_PATTERNS};

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
