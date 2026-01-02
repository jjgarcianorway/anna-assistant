//! Learning statistics display.
//!
//! Shows probe effectiveness, learned keywords, patterns, and health status.

use anna_shared::probe_learning::{LearningHealth, ProbeLearningStore, TrendDirection};
use anna_shared::ui::{
    colors, kv, print_footer, print_hint, print_section_header, print_step, print_title, symbols,
};
use anyhow::Result;

use super::recipe_analysis::show_recipe_stats;
use super::utils::{score_bar, truncate};

/// Handle learning command - show what Anna has learned
pub fn handle_learning() -> Result<()> {
    let store = ProbeLearningStore::load_with_decay();

    println!();
    print_title("Anna Learning Stats");
    println!();

    // v0.0.412: Show learned recipes first
    show_recipe_stats();

    if store.effectiveness.is_empty() && store.keyword_probes.is_empty() {
        print_hint("No probe learning data yet. Ask Anna some questions!");
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
            sorted_probes.sort_by(|a, b| {
                b.1.score
                    .partial_cmp(&a.1.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

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
                probes
                    .iter()
                    .take(3)
                    .map(|(p, _)| p.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            print_step(&format!(
                "\"{}\" {} {} (success: {})",
                keyword,
                symbols::ARROW,
                top_probes,
                stats.success_count
            ));
        }
        println!();
    }

    // v0.0.325: Show successful patterns count
    if !store.successful_patterns.is_empty() {
        let stats = store.learning_stats();
        print_section_header("patterns");
        kv(
            "successful",
            &format!(
                "{} (avg quality: {:.1}/5)",
                stats.successful_patterns, stats.avg_quality
            ),
        );
    }

    // Show negative patterns
    if !store.negative_patterns.is_empty() {
        kv("negative", &format!("{}", store.negative_patterns.len()));
        println!();

        for pattern in store.negative_patterns.iter().take(3) {
            print_step(&format!("\"{}\"", truncate(&pattern.query, 40)));
            print_hint(&format!("reason: {}", pattern.failure_reason));
        }
        println!();
    }

    // Summary
    let stats = store.learning_stats();
    print_section_header("summary");
    kv("queries_processed", &format!("{}", stats.total_queries));
    kv("keywords_learned", &format!("{}", stats.keywords_learned));
    kv(
        "successful_patterns",
        &format!("{}", stats.successful_patterns),
    );
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
    kv(
        "status",
        &format!(
            "{}{}{} ({:.0}% confidence)",
            health_color,
            health,
            colors::RESET,
            confidence * 100.0
        ),
    );

    if let Some(trend) = store.quality_trend() {
        let (trend_icon, trend_color) = match trend.trend {
            TrendDirection::Improving => ("↑", colors::OK),
            TrendDirection::Declining => ("↓", colors::ERR),
            TrendDirection::Stable => ("→", colors::DIM),
        };
        kv(
            "trend",
            &format!(
                "{}{}{} {} (was {:.1}, now {:.1})",
                trend_color,
                trend_icon,
                colors::RESET,
                trend.trend,
                trend.previous_avg,
                trend.current_avg
            ),
        );
    }

    if store.should_use_learning() {
        kv(
            "active",
            &format!(
                "{}yes{} - recommendations will be used",
                colors::OK,
                colors::RESET
            ),
        );
    } else {
        kv(
            "active",
            &format!("{}no{} - using defaults", colors::DIM, colors::RESET),
        );
    }

    println!();
    print_footer();
    Ok(())
}
