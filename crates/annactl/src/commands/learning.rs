//! Learning stats command (v0.0.328).
//!
//! Shows what Anna has learned from experience:
//! - Probe effectiveness per category
//! - Negative patterns (mistakes to avoid)
//! - Keyword associations (v0.0.325)
//! - Query recommendations test (v0.0.328)

use anna_shared::probe_learning::{ProbeLearningStore, QueryCategory};
use anna_shared::ui::colors;
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
fn show_query_recommendations(query: &str) -> Result<()> {
    let store = ProbeLearningStore::load_with_decay();
    let category = QueryCategory::from_query(query);

    println!();
    println!("{}Query Analysis{}", colors::HEADER, colors::RESET);
    println!();
    println!("  {}Query:{} \"{}\"", colors::DIM, colors::RESET, query);
    println!("  {}Category:{} {:?}", colors::DIM, colors::RESET, category);
    println!();

    // Get category-based recommendations
    let category_recs = store.get_recommended_probes(&category);
    if !category_recs.is_empty() {
        println!("{}Category-based Recommendations:{}", colors::BOLD, colors::RESET);
        for (probe_id, score) in category_recs.iter().take(5) {
            let score_color = if *score >= 0.7 { colors::OK }
                else if *score >= 0.5 { colors::WARN }
                else { colors::DIM };
            println!(
                "  {} {}{:.0}%{}",
                probe_id, score_color, score * 100.0, colors::RESET
            );
        }
        println!();
    } else {
        println!("{}No category-based recommendations yet{}", colors::DIM, colors::RESET);
        println!();
    }

    // Get keyword-based suggestions
    let keyword_suggestions = store.suggest_probes_for_query(query);
    if !keyword_suggestions.is_empty() {
        println!("{}Keyword-based Suggestions:{}", colors::BOLD, colors::RESET);
        for (probe_id, count) in keyword_suggestions.iter().take(5) {
            println!("  {} (keyword matches: {})", probe_id, count);
        }
        println!();
    } else {
        println!("{}No keyword-based suggestions yet{}", colors::DIM, colors::RESET);
        println!();
    }

    // Check for known bad combinations
    let probes: Vec<String> = category_recs.iter().map(|(p, _)| p.clone()).collect();
    if let Some(reason) = store.is_known_bad_combo(query, &probes) {
        println!(
            "{}Warning:{} Similar query had issues before: {}",
            colors::WARN, colors::RESET, reason
        );
        println!();
    }

    Ok(())
}

/// Handle learning command - show what Anna has learned
pub fn handle_learning() -> Result<()> {
    let store = ProbeLearningStore::load_with_decay();

    println!();
    println!("{}Anna Learning Stats{}", colors::HEADER, colors::RESET);
    println!();

    if store.effectiveness.is_empty() && store.keyword_probes.is_empty() {
        println!(
            "{}No learning data yet. Ask Anna some questions!{}",
            colors::DIM,
            colors::RESET
        );
        return Ok(());
    }

    // Show effectiveness per category
    if !store.effectiveness.is_empty() {
        println!("{}Probe Effectiveness by Category:{}", colors::BOLD, colors::RESET);
        println!();

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
                    "    {} {}{:.0}%{} [{}] uses:{} helpful:{} fails:{}",
                    probe_id,
                    score_color,
                    eff.score * 100.0,
                    colors::RESET,
                    bar,
                    eff.uses,
                    eff.helpful,
                    eff.failures
                );
            }
            println!();
        }
    }

    // v0.0.325: Show learned keywords
    if !store.keyword_probes.is_empty() {
        println!("{}Learned Keywords:{} {}", colors::BOLD, colors::RESET, store.keyword_probes.len());
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
                "  {}\"{}\"{}  →  {} (success: {})",
                colors::CYAN,
                keyword,
                colors::RESET,
                top_probes,
                stats.success_count
            );
        }
        println!();
    }

    // v0.0.325: Show successful patterns count
    if !store.successful_patterns.is_empty() {
        let stats = store.learning_stats();
        println!(
            "{}Successful Patterns:{} {} (avg quality: {:.1}/5)",
            colors::BOLD,
            colors::RESET,
            stats.successful_patterns,
            stats.avg_quality
        );
        println!();
    }

    // Show negative patterns
    if !store.negative_patterns.is_empty() {
        println!(
            "{}Negative Patterns (mistakes to avoid):{} {}",
            colors::BOLD,
            colors::RESET,
            store.negative_patterns.len()
        );
        println!();

        for pattern in store.negative_patterns.iter().take(5) {
            println!(
                "  {}Query:{} {}",
                colors::DIM,
                colors::RESET,
                truncate(&pattern.query, 50)
            );
            println!(
                "    {}Reason:{} {}",
                colors::DIM,
                colors::RESET,
                pattern.failure_reason
            );
            println!(
                "    {}Probes:{} {}",
                colors::DIM,
                colors::RESET,
                pattern.probes_used.join(", ")
            );
            println!();
        }
    }

    // Summary
    let stats = store.learning_stats();
    println!("{}Summary:{}", colors::BOLD, colors::RESET);
    println!(
        "  {} queries processed, {} keywords learned",
        stats.total_queries,
        stats.keywords_learned
    );
    println!(
        "  {} successful patterns, {} negative patterns",
        stats.successful_patterns,
        stats.negative_patterns
    );
    println!();

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
