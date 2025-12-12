//! Learning stats command (v0.0.412).
//!
//! Shows what Anna has learned from experience:
//! - Probe effectiveness per category
//! - Negative patterns (mistakes to avoid)
//! - Keyword associations (v0.0.325)
//! - Query recommendations test (v0.0.328)
//! - Health status and confidence (v0.0.334)
//! - v0.0.339: Use centralized UI helpers for consistency.
//! - v0.0.344: Use print_title() and print_footer() for consistency.
//! - v0.0.354: Use print_step() for arrow-prefixed lines.
//! - v0.0.406: Add suggest-recipes command for recipe candidate analysis.
//! - v0.0.412: Show learned recipes from RecipeStoreV2.

use anna_shared::probe_learning::{
    LearningHealth, ProbeLearningStore, QueryCategory, TrendDirection,
};
use anna_shared::recipe_store_v2::RecipeStoreV2;
use anna_shared::ticket_log::{calculate_stats, load_recent_tickets, TicketResult};
use anna_shared::ui::{
    colors, kv, print_footer, print_hint, print_label, print_section_header, print_step,
    print_title, symbols,
};
use anyhow::Result;
use std::collections::HashMap;

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
        kv(
            "learning",
            &format!(
                "{}Active{} ({:.0}% confidence)",
                colors::OK,
                colors::RESET,
                confidence * 100.0
            ),
        );
    } else {
        kv(
            "learning",
            &format!(
                "{}Inactive{} ({:.0}% - need 30%)",
                colors::WARN,
                colors::RESET,
                confidence * 100.0
            ),
        );
    }
    println!();

    // Get category-based recommendations
    let category_recs = store.get_recommended_probes(&category);
    print_section_header("category recommendations");
    if !category_recs.is_empty() {
        for (probe_id, score) in category_recs.iter().take(5) {
            let score_color = if *score >= 0.7 {
                colors::OK
            } else if *score >= 0.5 {
                colors::WARN
            } else {
                colors::DIM
            };
            print_step(&format!(
                "{} {}{:.0}%{}",
                probe_id,
                score_color,
                score * 100.0,
                colors::RESET
            ));
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
            print_step(&format!("{} (matches: {})", probe_id, count));
        }
    } else {
        print_hint("No keyword-based suggestions yet");
    }
    println!();

    // Check for known bad combinations
    let probes: Vec<String> = category_recs.iter().map(|(p, _)| p.clone()).collect();
    if let Some(reason) = store.is_known_bad_combo(query, &probes) {
        print_label(
            "warn",
            &format!("Similar query had issues: {}", reason),
            colors::WARN,
        );
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

/// v0.0.412: Show learned recipe statistics
fn show_recipe_stats() {
    let store = RecipeStoreV2::load();
    let stats = store.stats();

    print_section_header("learned recipes");

    if store.is_empty() {
        print_hint("No learned recipes yet - initialized with generic templates");
        println!();
        return;
    }

    kv("total recipes", &format!("{}", stats.total_recipes));
    kv("active", &format!("{}", stats.active_recipes));
    if stats.deprecated_recipes > 0 {
        kv(
            "deprecated",
            &format!(
                "{}{}{}",
                colors::WARN,
                stats.deprecated_recipes,
                colors::RESET
            ),
        );
    }
    kv("total uses", &format!("{}", stats.total_uses));
    kv(
        "success rate",
        &format!("{:.1}%", stats.overall_success_rate * 100.0),
    );

    // Show top recipes by use count
    let mut recipes: Vec<_> = store.recipes.values().collect();
    recipes.sort_by(|a, b| b.use_count.cmp(&a.use_count));
    let top_used: Vec<_> = recipes.iter().take(5).collect();

    if !top_used.is_empty() && top_used.iter().any(|r| r.use_count > 0) {
        println!();
        print_hint("most used:");
        for recipe in top_used {
            if recipe.use_count == 0 {
                continue;
            }
            let success_color = if recipe.success_rate() >= 0.8 {
                colors::OK
            } else if recipe.success_rate() >= 0.6 {
                colors::WARN
            } else {
                colors::ERR
            };
            print_step(&format!(
                "{} (uses: {}, success: {}{:.0}%{})",
                recipe.name,
                recipe.use_count,
                success_color,
                recipe.success_rate() * 100.0,
                colors::RESET
            ));
        }
    }

    println!();
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

/// v0.0.406: Suggest recipes based on ticket log patterns
/// Analyzes recent tickets to identify candidates for recipe creation
pub fn handle_suggest_recipes(limit: Option<usize>) -> Result<()> {
    let ticket_limit = limit.unwrap_or(100);
    let tickets = load_recent_tickets(ticket_limit);

    println!();
    print_title("Recipe Candidates");
    println!();

    if tickets.is_empty() {
        print_hint("No ticket logs found. Handle some queries first!");
        print_hint(&format!("Ticket logs are stored in ~/.anna/tickets/"));
        println!();
        return Ok(());
    }

    // Calculate overall stats first
    let stats = calculate_stats(&tickets);
    print_section_header("ticket summary");
    kv("total analyzed", &format!("{}", stats.total));
    kv(
        "success rate",
        &format!("{:.0}%", stats.success as f64 / stats.total as f64 * 100.0),
    );

    // Show handler distribution
    if let Some(recipe_count) = stats.by_handler.get("recipe") {
        kv(
            "handled by recipes",
            &format!(
                "{} ({:.0}%)",
                recipe_count,
                *recipe_count as f64 / stats.total as f64 * 100.0
            ),
        );
    }
    if let Some(llm_count) = stats.by_handler.get("llm") {
        kv(
            "handled by LLM",
            &format!(
                "{} ({:.0}%)",
                llm_count,
                *llm_count as f64 / stats.total as f64 * 100.0
            ),
        );
    }
    println!();

    // Group successful LLM-handled tickets by domain + intent
    let mut clusters: HashMap<(String, String), Vec<&anna_shared::ticket_log::TicketLog>> =
        HashMap::new();

    for ticket in &tickets {
        // Only consider successful LLM-handled tickets as recipe candidates
        if ticket.result != TicketResult::Success {
            continue;
        }
        if !ticket.handled_by.starts_with("llm:") && !ticket.handled_by.contains("specialist") {
            continue;
        }

        let key = (ticket.domain.clone(), ticket.intent.clone());
        clusters.entry(key).or_default().push(ticket);
    }

    if clusters.is_empty() {
        print_hint("No LLM-handled tickets found for recipe analysis.");
        println!();
        return Ok(());
    }

    // Sort clusters by count (most common patterns first)
    let mut sorted_clusters: Vec<_> = clusters.into_iter().collect();
    sorted_clusters.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    print_section_header("recipe candidates");
    println!();

    for ((domain, intent), tickets_in_cluster) in sorted_clusters.iter().take(10) {
        let count = tickets_in_cluster.len();
        if count < 2 {
            continue; // Skip single occurrences
        }

        // Find common patterns
        let mut probe_counts: HashMap<String, usize> = HashMap::new();
        let mut command_counts: HashMap<String, usize> = HashMap::new();
        let mut example_queries: Vec<&str> = vec![];

        for ticket in tickets_in_cluster {
            for probe in &ticket.probes {
                *probe_counts.entry(probe.id.clone()).or_default() += 1;
            }
            for cmd in &ticket.commands_run {
                *command_counts.entry(cmd.cmd.clone()).or_default() += 1;
            }
            if example_queries.len() < 3 {
                example_queries.push(&ticket.query);
            }
        }

        // Print cluster info
        println!("  {}{}::{}{}", colors::CYAN, domain, intent, colors::RESET);
        kv("  count", &format!("{} tickets", count));

        // Top probes used
        let mut sorted_probes: Vec<_> = probe_counts.into_iter().collect();
        sorted_probes.sort_by(|a, b| b.1.cmp(&a.1));
        if !sorted_probes.is_empty() {
            let top_probes: String = sorted_probes
                .iter()
                .take(3)
                .map(|(p, c)| format!("{} ({})", p, c))
                .collect::<Vec<_>>()
                .join(", ");
            kv("  common probes", &top_probes);
        }

        // Top commands used
        let mut sorted_cmds: Vec<_> = command_counts.into_iter().collect();
        sorted_cmds.sort_by(|a, b| b.1.cmp(&a.1));
        if !sorted_cmds.is_empty() {
            let top_cmds: String = sorted_cmds
                .iter()
                .take(2)
                .map(|(c, n)| format!("{} ({})", truncate(c, 30), n))
                .collect::<Vec<_>>()
                .join(", ");
            kv("  common commands", &top_cmds);
        }

        // Example queries
        print_hint("  example queries:");
        for q in example_queries {
            println!("    {} \"{}\"", symbols::BULLET, truncate(q, 50));
        }

        // Recommendation
        if count >= 5 {
            print_label("recommend", "High priority - create recipe", colors::OK);
        } else if count >= 3 {
            print_label(
                "recommend",
                "Medium priority - consider recipe",
                colors::WARN,
            );
        }

        println!();
    }

    // Summary
    let candidates: usize = sorted_clusters.iter().filter(|(_, t)| t.len() >= 2).count();
    let high_priority: usize = sorted_clusters.iter().filter(|(_, t)| t.len() >= 5).count();

    print_section_header("summary");
    kv("pattern clusters found", &format!("{}", candidates));
    kv("high priority candidates", &format!("{}", high_priority));
    println!();

    if high_priority > 0 {
        print_hint("Create recipes for high-priority patterns to reduce LLM usage.");
        print_hint("Recipe location: ~/.anna/recipes/authored/");
    }

    println!();
    print_footer();
    Ok(())
}
