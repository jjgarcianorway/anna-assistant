//! Stats display module for annactl (v0.0.75, v0.0.84, v0.0.85, v0.0.90, v0.0.91, v0.0.105).
//!
//! Provides RPG-style stats visualization with XP bars, levels, and titles.
//! v0.0.84: Enhanced with per-team breakdown, fun stats.
//! v0.0.85: Added installation date / tenure tracking.
//! v0.0.90: Added achievement badges.
//! v0.0.91: ASCII-style badges (no emojis) for Hollywood IT aesthetic.
//! v0.0.105: Added per-staff statistics, ticket tracking, recipe counts.
//! v0.0.107: Added staff performance leaderboard.

use anna_shared::achievements::{check_achievements, format_achievements};
use anna_shared::event_log::{AggregatedEvents, EventLog};
use anna_shared::recipe_matcher::recipe_count;
use anna_shared::staff_stats::StaffStats;
use anna_shared::stats::GlobalStats;
use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::{colors, HR};
use anna_shared::VERSION;

use crate::time_format::{format_date, format_tenure};

/// Print key-value pair
fn print_kv(key: &str, value: &str, width: usize) {
    println!("{:width$} {}", key, value, width = width);
}

/// Print stats display (v0.0.27, v0.0.67, v0.0.75: Enhanced RPG system)
pub fn print_stats_display(stats: &GlobalStats) {
    println!("\n{}annactl stats v{}{}", colors::HEADER, VERSION, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    // v0.0.75: Event log for RPG stats
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    if let Ok(agg) = event_log.aggregate() {
        if agg.total_requests > 0 {
            print_enhanced_rpg_stats(&agg);
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

/// Print enhanced RPG-style stats (v0.0.75, v0.0.84, v0.0.90)
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

    // Recipes - v0.0.105: Show actual recipe count from disk
    let total_recipes = recipe_count();
    if total_recipes > 0 || agg.recipes_learned > 0 || agg.recipes_used > 0 {
        println!(
            "  Recipes: {} stored, {} learned this session, {} used",
            total_recipes, agg.recipes_learned, agg.recipes_used
        );
    }

    // v0.0.105: Ticket tracking
    if let Ok(ticket_stats) = TicketTracker::for_user().stats() {
        if ticket_stats.total_tickets > 0 {
            println!(
                "  Tickets: {} total, {} resolved, {} escalated",
                ticket_stats.total_tickets,
                ticket_stats.resolved_tickets,
                ticket_stats.escalated_tickets
            );
        }
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

    // v0.0.107: Staff performance section
    print_staff_performance();

    // v0.0.84: Fun stats section
    print_fun_stats(agg);
}

/// v0.0.84/85: Print fun/interesting statistics
fn print_fun_stats(agg: &AggregatedEvents) {
    if agg.total_requests == 0 {
        return;
    }

    println!();
    println!("{}Fun Facts{}", colors::BOLD, colors::RESET);

    // v0.0.85: Anna since (installation date from first event)
    if agg.first_event_ts > 0 {
        let tenure = format_tenure(agg.first_event_ts);
        let since_date = format_date(agg.first_event_ts);
        println!(
            "  {} Anna since: {} ({})",
            bullet(), since_date, tenure
        );
    }

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

    // v0.0.86: Streaks
    if agg.current_streak > 1 {
        println!(
            "  {} {}🔥 {} day streak!{} Keep it going!",
            bullet(), colors::OK, agg.current_streak, colors::RESET
        );
    }
    if agg.best_streak > agg.current_streak && agg.best_streak > 2 {
        println!(
            "  {} Best streak: {} days",
            bullet(), agg.best_streak
        );
    }

    // Active days
    if agg.active_days > 1 {
        println!(
            "  {} Active on {} different days",
            bullet(), agg.active_days
        );
    }

    // Lucky team
    if let Some(ref team) = agg.lucky_team {
        if agg.lucky_team_rate >= 0.9 {
            println!(
                "  {} {}Lucky team:{} {} ({:.0}% success rate)",
                bullet(), colors::OK, colors::RESET, team, agg.lucky_team_rate * 100.0
            );
        }
    }

    // v0.0.90: Achievements
    print_achievements(agg);
}

/// v0.0.90: Print achievement badges
fn print_achievements(agg: &AggregatedEvents) {
    let achievements = check_achievements(agg);
    let unlocked: Vec<_> = achievements.iter().filter(|a| a.unlocked).collect();

    if unlocked.is_empty() {
        return;
    }

    println!();
    println!("{}Achievements{}", colors::BOLD, colors::RESET);

    // Show ASCII badge summary line
    let badge_line = format_achievements(&achievements, 10);
    if !badge_line.is_empty() {
        println!("  {}", badge_line);
    }

    // Show notable achievements with descriptions
    for ach in unlocked.iter().filter(|a| is_notable(a.id)).take(3) {
        println!("  {} {} - {}", ach.badge, ach.name, ach.description);
    }
}

fn is_notable(id: &str) -> bool {
    matches!(id, "hundred_queries" | "five_hundred" | "streak_7" | "streak_30" |
        "perfect_10" | "no_failures" | "all_teams" | "recipe_master" | "month_old")
}

/// v0.0.107: Print staff performance leaderboard
fn print_staff_performance() {
    let stats = StaffStats::load();
    if stats.total_tickets() == 0 {
        return;
    }

    println!();
    println!("{}Staff Performance{}", colors::BOLD, colors::RESET);

    let top = stats.top_performers(5);
    if top.is_empty() {
        return;
    }

    println!(
        "  {:18} {:>8} {:>8} {:>8} {:>10}",
        "Staff", "Tickets", "Resolved", "Rate", "Avg Time"
    );
    println!("  {}", "-".repeat(55));

    for (person_id, metrics) in top {
        // Extract display name from person_id (e.g., "desktop_jr_sofia" -> "Sofia")
        let name = person_id
            .split('_')
            .last()
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                    None => s.to_string(),
                }
            })
            .unwrap_or_else(|| person_id.clone());

        let rate_color = if metrics.success_rate() >= 80.0 {
            colors::OK
        } else if metrics.success_rate() >= 50.0 {
            colors::WARN
        } else {
            colors::ERR
        };

        println!(
            "  {:18} {:>8} {:>8} {}{:>7.0}%{} {:>9}ms",
            name,
            metrics.tickets_handled,
            metrics.tickets_resolved,
            rate_color,
            metrics.success_rate(),
            colors::RESET,
            metrics.avg_time_ms()
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
