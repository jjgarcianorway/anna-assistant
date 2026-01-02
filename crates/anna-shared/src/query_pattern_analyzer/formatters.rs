//! Query Pattern Analyzer Formatters
//!
//! Formatting functions for displaying pattern analyzer data.

use super::analyzer::QueryPatternAnalyzer;

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
