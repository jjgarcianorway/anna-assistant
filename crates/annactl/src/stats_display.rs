//! Stats display module for annactl (v0.0.67).
//!
//! Provides RPG-style stats visualization with XP bars and titles.

use anna_shared::stats::GlobalStats;
use anna_shared::stats_store::{AggregatedStats, StatsStore};
use anna_shared::ui::{colors, HR};
use anna_shared::VERSION;

/// Print key-value pair
fn print_kv(key: &str, value: &str, width: usize) {
    println!("{:width$} {}", key, value, width = width);
}

/// Print stats display (v0.0.27, v0.0.67: RPG system)
pub fn print_stats_display(stats: &GlobalStats) {
    println!("\n{}annactl stats v{}{}", colors::HEADER, VERSION, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    // v0.0.67: Try to load RPG stats from local store
    let store = StatsStore::default_location();
    if let Ok(agg) = store.aggregate() {
        print_rpg_stats(&agg);
        println!();
    }

    let kw = 15; // key width

    // Global summary
    print_kv("total_requests", &stats.total_requests.to_string(), kw);
    print_kv(
        "success_rate",
        &format!("{:.1}%", stats.overall_success_rate() * 100.0),
        kw,
    );
    print_kv(
        "avg_reliability",
        &format!("{:.0}", stats.overall_avg_score()),
        kw,
    );

    if let Some(team) = stats.most_consulted_team {
        print_kv("top_team", &team.to_string(), kw);
    }

    println!();
    println!("{}Per-Team Statistics:{}", colors::BOLD, colors::RESET);
    println!(
        "  {:12} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Team", "Total", "Success", "Failed", "AvgRnd", "AvgScore"
    );
    println!("{}{}{}", colors::DIM, "-".repeat(60), colors::RESET);

    for ts in &stats.by_team {
        if ts.tickets_total > 0 {
            let success_color = if ts.success_rate() >= 0.8 {
                colors::OK
            } else if ts.success_rate() >= 0.5 {
                colors::WARN
            } else {
                colors::ERR
            };
            println!(
                "  {:12} {:>8} {}{:>8}{} {:>8} {:>8.1} {:>8.0}",
                ts.team,
                ts.tickets_total,
                success_color,
                ts.tickets_verified,
                colors::RESET,
                ts.tickets_failed,
                ts.avg_rounds,
                ts.avg_reliability_score,
            );
        }
    }

    // Show teams with no activity
    let inactive: Vec<_> = stats.by_team.iter().filter(|ts| ts.tickets_total == 0).collect();
    if !inactive.is_empty() {
        println!();
        println!(
            "{}Inactive teams:{} {}",
            colors::DIM,
            colors::RESET,
            inactive.iter().map(|t| t.team.to_string()).collect::<Vec<_>>().join(", ")
        );
    }

    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}

/// Print RPG-style stats (v0.0.67)
fn print_rpg_stats(agg: &AggregatedStats) {
    if agg.total_requests == 0 {
        return;
    }

    let xp = agg.calculate_xp();
    let title = agg.xp_title();

    // XP bar visualization
    let bar_width = 30;
    let filled = (xp as usize * bar_width) / 100;
    let empty = bar_width - filled;
    let bar = format!(
        "[{}{}{}{}]",
        colors::OK,
        "=".repeat(filled),
        colors::DIM,
        "-".repeat(empty)
    );

    println!("{}Service Desk Profile{}", colors::BOLD, colors::RESET);
    println!();
    println!("  {} {}{}{}", bar, colors::OK, xp, colors::RESET);
    println!("  {}{}{}", colors::CYAN, title, colors::RESET);
    println!();
    println!(
        "  Cases: {}   Success: {:.0}%   Avg Reliability: {:.0}",
        agg.total_requests,
        agg.success_rate() * 100.0,
        agg.average_reliability
    );

    if let Some(ref team) = agg.most_consulted_team {
        println!("  Most consulted: {}", team);
    }

    if agg.escalated_count > 0 {
        println!("  Escalations: {}", agg.escalated_count);
    }
}
