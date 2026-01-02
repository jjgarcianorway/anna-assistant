//! Recipe learning and suggestion analysis.
//!
//! v0.0.406: Suggest recipes based on ticket log patterns.
//! v0.0.412: Show learned recipe statistics.

use anna_shared::recipe_store_v2::RecipeStoreV2;
use anna_shared::ticket_log::{calculate_stats, load_recent_tickets, TicketResult};
use anna_shared::ui::{
    colors, kv, print_footer, print_hint, print_label, print_section_header, print_step,
    print_title, symbols,
};
use anyhow::Result;
use std::collections::HashMap;

use super::utils::truncate;

/// v0.0.412: Show learned recipe statistics
pub fn show_recipe_stats() {
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
