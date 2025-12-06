//! Stats display module for annactl (v0.0.75, v0.0.84).
//!
//! Provides RPG-style stats visualization with XP bars, levels, and titles.
//! v0.0.84: Enhanced with per-team breakdown, fun stats, installation date.

use anna_shared::event_log::{AggregatedEvents, EventLog};
use anna_shared::stats::GlobalStats;
use anna_shared::stats_store::{AggregatedStats, StatsStore};
use anna_shared::ui::{colors, HR};
use anna_shared::VERSION;

/// Print key-value pair
fn print_kv(key: &str, value: &str, width: usize) {
    println!("{:width$} {}", key, value, width = width);
}

/// Print stats display (v0.0.27, v0.0.67, v0.0.75: Enhanced RPG system)
pub fn print_stats_display(stats: &GlobalStats) {
    println!("\n{}annactl stats v{}{}", colors::HEADER, VERSION, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    // v0.0.75: Try event log first (new system)
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    if let Ok(agg) = event_log.aggregate() {
        if agg.total_requests > 0 {
            print_enhanced_rpg_stats(&agg);
            println!();
        }
    } else {
        // Fallback to v0.0.67 stats store
        let store = StatsStore::default_location();
        if let Ok(agg) = store.aggregate() {
            print_rpg_stats(&agg);
            println!();
        }
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

/// Print RPG-style stats (v0.0.67 - legacy)
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

/// Print enhanced RPG-style stats (v0.0.75, v0.0.84)
fn print_enhanced_rpg_stats(agg: &AggregatedEvents) {
    println!("{}Service Desk Profile{}", colors::BOLD, colors::RESET);
    println!();

    // Level and XP display
    let xp_for_next = xp_for_next_level(agg.level);
    let xp_at_level_start = xp_at_level_start(agg.level);
    let progress = if xp_for_next > xp_at_level_start {
        ((agg.xp.saturating_sub(xp_at_level_start)) as f32
            / (xp_for_next - xp_at_level_start) as f32
            * 100.0) as u8
    } else {
        100
    };

    // Progress bar for current level
    let bar_width = 20;
    let filled = (progress as usize * bar_width) / 100;
    let empty = bar_width.saturating_sub(filled);
    let bar = format!(
        "[{}{}{}{}{}]",
        colors::OK,
        "█".repeat(filled),
        colors::DIM,
        "░".repeat(empty),
        colors::RESET
    );

    println!(
        "  {}Level {}{} {}",
        colors::BOLD,
        agg.level,
        colors::RESET,
        bar
    );
    println!("  {}{}{}", colors::CYAN, agg.title, colors::RESET);
    println!(
        "  {}XP: {}/{} to next level{}",
        colors::DIM,
        agg.xp,
        xp_for_next,
        colors::RESET
    );
    println!();

    // Stats summary
    let success_rate = if agg.total_requests > 0 {
        agg.verified_count as f32 / agg.total_requests as f32 * 100.0
    } else {
        0.0
    };

    println!(
        "  Cases: {}   Verified: {}   Failed: {}",
        agg.total_requests, agg.verified_count, agg.failed_count
    );
    println!(
        "  Success: {}{:.0}%{}   Avg Reliability: {:.0}",
        if success_rate >= 80.0 {
            colors::OK
        } else if success_rate >= 60.0 {
            colors::WARN
        } else {
            colors::ERR
        },
        success_rate,
        colors::RESET,
        agg.avg_reliability
    );

    // Recipes
    if agg.recipes_learned > 0 || agg.recipes_used > 0 {
        println!(
            "  Recipes: {} learned, {} used",
            agg.recipes_learned, agg.recipes_used
        );
    }

    // Escalations
    if agg.escalation_count > 0 {
        print!("  Escalations: {}", agg.escalation_count);
        if let Some(ref team) = agg.most_escalated_team {
            print!(" (most: {})", team);
        }
        println!();
    }

    // Performance stats
    if agg.total_requests > 0 {
        println!();
        println!(
            "  {}Avg response: {:.0}ms   Min: {}ms   Max: {}ms{}",
            colors::DIM,
            agg.avg_duration_ms,
            agg.min_duration_ms,
            agg.max_duration_ms,
            colors::RESET
        );
    }

    // v0.0.84: Fun stats section
    print_fun_stats(agg);
}

/// v0.0.84: Print fun/interesting statistics
fn print_fun_stats(agg: &AggregatedEvents) {
    if agg.total_requests == 0 {
        return;
    }

    println!();
    println!("{}Fun Facts{}", colors::BOLD, colors::RESET);

    // Most consulted team
    if let Some((team, count)) = agg.by_team.iter().max_by_key(|(_, c)| *c) {
        let pct = (*count as f32 / agg.total_requests as f32 * 100.0) as u32;
        println!(
            "  {} Most consulted: {} ({}% of cases)",
            bullet(), team, pct
        );
    }

    // Least consulted team (if we have multiple teams)
    if agg.by_team.len() > 1 {
        if let Some((team, count)) = agg.by_team.iter()
            .filter(|(_, c)| **c > 0)
            .min_by_key(|(_, c)| *c)
        {
            println!(
                "  {} Least consulted: {} ({} case{})",
                bullet(), team, count, if *count == 1 { "" } else { "s" }
            );
        }
    }

    // Speed records
    if agg.max_duration_ms > 0 && agg.min_duration_ms < u64::MAX {
        println!(
            "  {} Fastest answer: {}ms",
            bullet(), agg.min_duration_ms
        );
        if agg.max_duration_ms > 5000 {
            println!(
                "  {} Longest research: {:.1}s (that was a tough one!)",
                bullet(), agg.max_duration_ms as f64 / 1000.0
            );
        }
    }

    // Anna managed on her own (fast path)
    // We'd need to track this separately, but we can estimate from timeouts
    if agg.timeout_count == 0 && agg.total_requests > 5 {
        println!(
            "  {} {}Zero timeouts!{} Anna always came through.",
            bullet(), colors::OK, colors::RESET
        );
    }

    // High performer
    if agg.avg_reliability >= 85.0 && agg.total_requests >= 10 {
        println!(
            "  {} {}High performer!{} Avg reliability of {:.0}%",
            bullet(), colors::OK, colors::RESET, agg.avg_reliability
        );
    }
}

fn bullet() -> &'static str {
    "›"
}

/// Get XP required for next level
fn xp_for_next_level(level: u32) -> u64 {
    match level {
        1 => 100,
        2 => 300,
        3 => 600,
        4 => 1000,
        5 => 2000,
        6 => 4000,
        7 => 8000,
        8 => 16000,
        9 => 32000,
        10 => 64000,
        _ => 100000,
    }
}

/// Get XP at the start of a level
fn xp_at_level_start(level: u32) -> u64 {
    match level {
        1 => 0,
        2 => 100,
        3 => 300,
        4 => 600,
        5 => 1000,
        6 => 2000,
        7 => 4000,
        8 => 8000,
        9 => 16000,
        10 => 32000,
        _ => 64000,
    }
}
