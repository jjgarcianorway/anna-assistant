//! Learning stats command (v0.0.344).
//!
//! Shows what Anna has learned from experience:
//! - Probe effectiveness per category
//! - Negative patterns (mistakes to avoid)
//! - Keyword associations (v0.0.325)
//! - Query recommendations test (v0.0.328)
//! - Health status and confidence (v0.0.334)
//! - v0.0.339: Use centralized UI helpers for consistency.
//! - v0.0.344: Use print_title() and print_footer() for consistency.

use anna_shared::probe_learning::{LearningHealth, ProbeLearningStore, QueryCategory, TrendDirection};
use anna_shared::ui::{colors, kv, print_footer, print_hint, print_label, print_section_header, print_title, symbols};
use anyhow::Result;

/// Handle learning command - show what Anna has learned
/// v0.0.328: Optional query parameter to test recommendations
pub fn handle_learning_with_query(query: Option<&str>) -> Result<()> {
    // If query provided, show recommendations for it
    if let Some(q) = query {
        return show_query_recommendations(q);
    }
    handle_learning()
}

/// v0.0.328: Show what probes Anna would recommend for a query
/// v0.0.334: Shows whether learning is active for this query
fn show_query_recommendations(query: &str) -> Result<()> {
    let store = ProbeLearningStore::load_with_decay();
    let category = QueryCategory::from_query(query);

    println!();
    print_title("Query Analysis");
    println!();

    print_section_header("input");
    kv("query", &format!("\"{}\"", query));
    kv("category", &format!("{:?}", category));

    // v0.0.334: Show if learning would be used for this query
    let confidence = store.confidence_factor();
    if store.should_use_learning() {
        kv("learning", &format!("{}Active{} ({:.0}% confidence)", colors::OK, colors::RESET, confidence * 100.0));
    } else {
        kv("learning", &format!("{}Inactive{} ({:.0}% - need 30%)", colors::WARN, colors::RESET, confidence * 100.0));
    }
    println!();

    // Get category-based recommendations
    let category_recs = store.get_recommended_probes(&category);
    print_section_header("category recommendations");
    if !category_recs.is_empty() {
        for (probe_id, score) in category_recs.iter().take(5) {
            let score_color = if *score >= 0.7 { colors::OK }
                else if *score >= 0.5 { colors::WARN }
                else { colors::DIM };
            println!("  {} {} {}{:.0}%{}", symbols::ARROW, probe_id, score_color, score * 100.0, colors::RESET);
        }
    } else {
        print_hint("No category-based recommendations yet");
    }
    println!();

    // Get keyword-based suggestions
    let keyword_suggestions = store.suggest_probes_for_query(query);
    print_section_header("keyword suggestions");
    if !keyword_suggestions.is_empty() {
        for (probe_id, count) in keyword_suggestions.iter().take(5) {
            println!("  {} {} (matches: {})", symbols::ARROW, probe_id, count);
        }
    } else {
        print_hint("No keyword-based suggestions yet");
    }
    println!();

    // Check for known bad combinations
    let probes: Vec<String> = category_recs.iter().map(|(p, _)| p.clone()).collect();
    if let Some(reason) = store.is_known_bad_combo(query, &probes) {
        print_label("warn", &format!("Similar query had issues: {}", reason), colors::WARN);
        println!();
    }

    print_footer();
    Ok(())
}

/// Handle learning command - show what Anna has learned
pub fn handle_learning() -> Result<()> {
    let store = ProbeLearningStore::load_with_decay();

    println!();
    print_title("Anna Learning Stats");
    println!();

    if store.effectiveness.is_empty() && store.keyword_probes.is_empty() {
        print_hint("No learning data yet. Ask Anna some questions!");
        println!();
        return Ok(());
    }

    // Show effectiveness per category
    if !store.effectiveness.is_empty() {
        print_section_header("probe effectiveness");
        for (category, probes) in &store.effectiveness {
            if probes.is_empty() {
                continue;
            }

            println!("  {}{:?}:{}", colors::CYAN, category, colors::RESET);

            // Sort by score descending
            let mut sorted_probes: Vec<_> = probes.iter().collect();
            sorted_probes.sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap_or(std::cmp::Ordering::Equal));

            for (probe_id, eff) in sorted_probes.iter().take(5) {
                let score_color = if eff.score >= 0.7 {
                    colors::OK
                } else if eff.score >= 0.5 {
                    colors::WARN
                } else {
                    colors::ERR
                };

                let bar = score_bar(eff.score, 10);
                println!(
                    "    {} {}{:.0}%{} [{}] uses:{} ok:{} fail:{}",
                    probe_id, score_color, eff.score * 100.0, colors::RESET,
                    bar, eff.uses, eff.helpful, eff.failures
                );
            }
        }
        println!();
    }

    // v0.0.325: Show learned keywords
    if !store.keyword_probes.is_empty() {
        print_section_header("learned keywords");
        kv("total", &format!("{}", store.keyword_probes.len()));
        println!();

        // Sort by success count
        let mut sorted_keywords: Vec<_> = store.keyword_probes.iter().collect();
        sorted_keywords.sort_by(|a, b| b.1.success_count.cmp(&a.1.success_count));

        for (keyword, stats) in sorted_keywords.iter().take(10) {
            let top_probes: String = {
                let mut probes: Vec<_> = stats.effective_probes.iter().collect();
                probes.sort_by(|a, b| b.1.cmp(a.1));
                probes.iter().take(3).map(|(p, _)| p.as_str()).collect::<Vec<_>>().join(", ")
            };
            println!(
                "  {} \"{}\" {} {} (success: {})",
                symbols::ARROW, keyword, symbols::ARROW, top_probes, stats.success_count
            );
        }
        println!();
    }

    // v0.0.325: Show successful patterns count
    if !store.successful_patterns.is_empty() {
        let stats = store.learning_stats();
        print_section_header("patterns");
        kv("successful", &format!("{} (avg quality: {:.1}/5)", stats.successful_patterns, stats.avg_quality));
    }

    // Show negative patterns
    if !store.negative_patterns.is_empty() {
        kv("negative", &format!("{}", store.negative_patterns.len()));
        println!();

        for pattern in store.negative_patterns.iter().take(3) {
            println!("  {} \"{}\"", symbols::ARROW, truncate(&pattern.query, 40));
            println!("    {}reason:{} {}", colors::DIM, colors::RESET, pattern.failure_reason);
        }
        println!();
    }

    // Summary
    let stats = store.learning_stats();
    print_section_header("summary");
    kv("queries_processed", &format!("{}", stats.total_queries));
    kv("keywords_learned", &format!("{}", stats.keywords_learned));
    kv("successful_patterns", &format!("{}", stats.successful_patterns));
    kv("negative_patterns", &format!("{}", stats.negative_patterns));
    println!();

    // v0.0.334: Health status and confidence
    let health = store.health_status();
    let confidence = store.confidence_factor();
    let health_color = match health {
        LearningHealth::Excellent => colors::OK,
        LearningHealth::Good => colors::OK,
        LearningHealth::Developing => colors::WARN,
        LearningHealth::NeedsAttention => colors::ERR,
        LearningHealth::Insufficient => colors::DIM,
    };

    print_section_header("health");
    kv("status", &format!("{}{}{} ({:.0}% confidence)", health_color, health, colors::RESET, confidence * 100.0));

    if let Some(trend) = store.quality_trend() {
        let (trend_icon, trend_color) = match trend.trend {
            TrendDirection::Improving => ("↑", colors::OK),
            TrendDirection::Declining => ("↓", colors::ERR),
            TrendDirection::Stable => ("→", colors::DIM),
        };
        kv("trend", &format!("{}{}{} {} (was {:.1}, now {:.1})",
            trend_color, trend_icon, colors::RESET, trend.trend, trend.previous_avg, trend.current_avg));
    }

    if store.should_use_learning() {
        kv("active", &format!("{}yes{} - recommendations will be used", colors::OK, colors::RESET));
    } else {
        kv("active", &format!("{}no{} - using defaults", colors::DIM, colors::RESET));
    }

    println!();
    print_footer();
    Ok(())
}

/// Create a visual score bar
fn score_bar(score: f32, width: usize) -> String {
    let filled = (score * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "{}{}{}{}{}",
        colors::OK,
        "█".repeat(filled),
        colors::DIM,
        "░".repeat(empty),
        colors::RESET
    )
}

/// Truncate string with ellipsis
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
