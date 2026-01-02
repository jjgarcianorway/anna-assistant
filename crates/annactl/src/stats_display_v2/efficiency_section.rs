//! Efficiency stats section (v0.0.406) - Recipe vs LLM stats.

use anna_shared::ticket_log::{calculate_stats, load_recent_tickets};
use anna_shared::ui::{colors, kv, print_section_header};

/// v0.0.407: Print truthful stats section
pub fn print_efficiency_section() {
    let tickets = load_recent_tickets(100);

    // Only show if there's data
    if tickets.is_empty() {
        return;
    }

    let stats = calculate_stats(&tickets);

    println!();
    print_section_header("outcomes");

    // v0.0.407: Truthful outcome counts
    let total = stats.total;
    if total > 0 {
        kv("total_tickets", &format!("{}", total));

        // Success rate with color coding
        let rate_color = if stats.success_rate >= 80.0 {
            colors::OK
        } else if stats.success_rate >= 50.0 {
            colors::WARN
        } else {
            colors::ERR
        };
        kv(
            "resolved",
            &format!(
                "{}{}{} ({}{:.0}%{})",
                colors::OK,
                stats.success,
                colors::RESET,
                rate_color,
                stats.success_rate,
                colors::RESET
            ),
        );

        if stats.failed > 0 {
            kv(
                "failed",
                &format!("{}{}{}", colors::ERR, stats.failed, colors::RESET),
            );
        }

        // v0.0.407: Show LLM failures separately
        if stats.llm_failed > 0 {
            kv(
                "llm_failed",
                &format!(
                    "{}{}{} (timeout/parse errors)",
                    colors::WARN,
                    stats.llm_failed,
                    colors::RESET
                ),
            );
        }

        if stats.escalated > 0 {
            kv("escalated", &format!("{}", stats.escalated));
        }
    }

    println!();
    print_section_header("handlers");

    // Handler breakdown
    let recipe_count = stats.by_handler.get("recipe").copied().unwrap_or(0);
    let llm_count = stats.by_handler.get("llm").copied().unwrap_or(0);
    let det_count = stats.by_handler.get("deterministic").copied().unwrap_or(0);

    if total > 0 {
        if recipe_count > 0 {
            let recipe_pct = recipe_count as f64 / total as f64 * 100.0;
            kv(
                "recipes",
                &format!(
                    "{}{}{} ({:.0}%)",
                    colors::OK,
                    recipe_count,
                    colors::RESET,
                    recipe_pct
                ),
            );
        }
        if llm_count > 0 {
            let llm_pct = llm_count as f64 / total as f64 * 100.0;
            kv(
                "llm",
                &format!(
                    "{}{}{} ({:.0}%)",
                    colors::WARN,
                    llm_count,
                    colors::RESET,
                    llm_pct
                ),
            );
        }
        if det_count > 0 {
            let det_pct = det_count as f64 / total as f64 * 100.0;
            kv(
                "deterministic",
                &format!(
                    "{}{}{} ({:.0}%)",
                    colors::DIM,
                    det_count,
                    colors::RESET,
                    det_pct
                ),
            );
        }

        // LLM savings indicator
        let savings = recipe_count + det_count;
        if savings > 0 {
            let savings_pct = savings as f64 / total as f64 * 100.0;
            kv(
                "llm_avoided",
                &format!(
                    "{}{:.0}%{} handled without LLM",
                    colors::OK,
                    savings_pct,
                    colors::RESET
                ),
            );
        }
    }

    // Average metrics (only from answered tickets)
    if stats.avg_duration_ms > 0 || stats.avg_reliability > 0 {
        println!();
        print_section_header("metrics");

        if stats.avg_duration_ms > 0 {
            kv(
                "avg_response",
                &format!(
                    "{:.1}s (answered only)",
                    stats.avg_duration_ms as f64 / 1000.0
                ),
            );
        }
        if stats.avg_reliability > 0 {
            let rel_color = if stats.avg_reliability >= 80 {
                colors::OK
            } else if stats.avg_reliability >= 60 {
                colors::WARN
            } else {
                colors::DIM
            };
            kv(
                "avg_reliability",
                &format!("{}{}%{}", rel_color, stats.avg_reliability, colors::RESET),
            );
        }
    }
}
