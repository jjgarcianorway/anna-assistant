//! Stats category display functions (v0.0.464).
//!
//! Modular category views for `annactl stats <category>`:
//! - rpg: RPG progression stats
//! - topics: Domain and intent breakdown
//! - repeated: Repeated questions

use anna_shared::event_log::EventLog;
use anna_shared::stats::GlobalStats;
use anna_shared::ticket_log::{calculate_stats, load_recent_tickets};
use anna_shared::ui::{colors, kv, kv_colored, print_footer, print_section_header, print_title};

use crate::stats_display_v2::{
    print_efficiency_section, print_learning_section, print_stats_display_v2,
};

/// v0.0.464: Print stats with optional category filter
/// Categories: rpg, learning, outcomes, handlers, topics, repeated
pub fn print_stats_with_category(stats: &GlobalStats, category: Option<&str>) {
    match category {
        None => print_stats_display_v2(stats),
        Some(cat) => match cat.to_lowercase().as_str() {
            "rpg" => print_rpg_only(),
            "learning" => {
                print_title("Anna Service Desk | Learning Stats");
                print_learning_section();
                print_footer();
            }
            "outcomes" | "handlers" => {
                print_title("Anna Service Desk | Ticket Outcomes");
                print_efficiency_section();
                print_footer();
            }
            "topics" => print_topics_section(),
            "repeated" => print_repeated_section(),
            _ => {
                println!("Unknown category: {}", cat);
                println!("Available: rpg, learning, outcomes, handlers, topics, repeated");
            }
        },
    }
}

/// v0.0.464: Print RPG stats only
fn print_rpg_only() {
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    let agg = event_log.aggregate().ok();

    print_title("Anna Service Desk | RPG Stats");

    if let Some(ref agg) = agg {
        if agg.total_requests > 0 {
            println!();
            print_section_header("rpg");

            let xp_bar = anna_shared::ui::progress_bar(agg.xp as f32 / 100.0, 20);
            kv(
                "xp",
                &format!(
                    "{}{}/100{}  {}",
                    colors::CYAN,
                    agg.xp,
                    colors::RESET,
                    xp_bar
                ),
            );
            kv(
                "level",
                &format!(
                    "{} - {}{}{}",
                    agg.level,
                    colors::BOLD,
                    agg.title,
                    colors::RESET
                ),
            );

            if agg.first_event_ts > 0 {
                let install_date = chrono::DateTime::from_timestamp(agg.first_event_ts as i64, 0)
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "-".to_string());
                kv("installed", &install_date);
            }

            if agg.anna_solo_count > 0 {
                let solo_pct = if agg.total_requests > 0 {
                    agg.anna_solo_count as f32 / agg.total_requests as f32 * 100.0
                } else {
                    0.0
                };
                kv(
                    "anna_solo",
                    &format!(
                        "{} ({:.0}% without specialists)",
                        agg.anna_solo_count, solo_pct
                    ),
                );
            }

            if agg.recipes_learned > 0 {
                kv("recipes_learned", &format!("{}", agg.recipes_learned));
            }

            if let Some(ref team) = agg.most_consulted_team {
                kv("most_consulted", team);
            }

            if agg.min_duration_ms > 0 || agg.max_duration_ms > 0 {
                kv(
                    "response_times",
                    &format!(
                        "fastest: {:.1}s, longest: {:.1}s",
                        agg.min_duration_ms as f64 / 1000.0,
                        agg.max_duration_ms as f64 / 1000.0
                    ),
                );
            }

            if agg.best_streak > 0 {
                kv(
                    "streaks",
                    &format!(
                        "current: {} days, best: {} days",
                        agg.current_streak, agg.best_streak
                    ),
                );
            }

            if agg.avg_interactions > 0.0 {
                kv(
                    "avg_interactions",
                    &format!("{:.1} (max: {})", agg.avg_interactions, agg.max_interactions),
                );
            }
        } else {
            println!("  No activity recorded yet.");
        }
    } else {
        println!("  No activity recorded yet.");
    }

    print_footer();
}

/// v0.0.464: Print topics breakdown (most asked domains)
fn print_topics_section() {
    let tickets = load_recent_tickets(100);

    print_title("Anna Service Desk | Topics");

    if tickets.is_empty() {
        println!("  No tickets recorded yet.");
        print_footer();
        return;
    }

    let stats = calculate_stats(&tickets);

    println!();
    print_section_header("by domain");

    let mut domains: Vec<_> = stats.by_domain.iter().collect();
    domains.sort_by(|a, b| b.1.cmp(a.1));

    for (domain, count) in domains.iter().take(10) {
        let pct = **count as f64 / stats.total as f64 * 100.0;
        kv(domain, &format!("{} ({:.0}%)", count, pct));
    }

    if let Some(ref top) = stats.top_topic {
        println!();
        kv_colored("most_asked_topic", top, colors::CYAN);
    }

    println!();
    print_section_header("by intent");

    let mut intents: Vec<_> = stats.by_intent.iter().collect();
    intents.sort_by(|a, b| b.1.cmp(a.1));

    for (intent, count) in intents.iter().take(10) {
        let pct = **count as f64 / stats.total as f64 * 100.0;
        kv(intent, &format!("{} ({:.0}%)", count, pct));
    }

    print_footer();
}

/// v0.0.464: Print repeated questions
fn print_repeated_section() {
    let tickets = load_recent_tickets(100);

    print_title("Anna Service Desk | Repeated Questions");

    if tickets.is_empty() {
        println!("  No tickets recorded yet.");
        print_footer();
        return;
    }

    let stats = calculate_stats(&tickets);

    println!();
    print_section_header("repeated queries");

    if stats.repeated_queries.is_empty() {
        println!("  No repeated questions detected.");
    } else {
        let mut repeated: Vec<_> = stats.repeated_queries.iter().collect();
        repeated.sort_by(|a, b| b.1.cmp(a.1));

        for (query, count) in repeated.iter().take(10) {
            // Truncate long queries
            let display_query = if query.len() > 40 {
                format!("{}...", &query[..37])
            } else {
                query.to_string()
            };
            kv(&display_query, &format!("{}x", count));
        }
    }

    println!();
    println!(
        "  {}Tip: Repeated questions are recipe candidates!{}",
        colors::DIM,
        colors::RESET
    );

    print_footer();
}
