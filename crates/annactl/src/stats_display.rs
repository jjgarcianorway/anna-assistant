//! Stats display module for annactl.
//! v0.0.119: Clean, focused stats display.

use anna_shared::achievements::{check_achievements, format_achievements};
use anna_shared::event_log::{AggregatedEvents, EventLog};
use anna_shared::recipe_matcher::recipe_count;
use anna_shared::staff_stats::StaffStats;
use anna_shared::stats::GlobalStats;
use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::{colors, HR};

use crate::time_format::{format_date, format_tenure};

/// v0.0.118: Clean stats display
pub fn print_stats_display(stats: &GlobalStats) {
    println!();
    println!("{}Anna Service Desk Stats{}", colors::HEADER, colors::RESET);
    println!("{}", HR);

    // Load event log for profile stats
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    let agg = event_log.aggregate().ok();

    // === PROFILE SECTION ===
    if let Some(ref agg) = agg {
        if agg.total_requests > 0 {
            print_profile(agg);
        }
    }

    // === ACTIVITY SUMMARY ===
    println!();
    println!("{}Activity{}", colors::BOLD, colors::RESET);

    println!("  Cases handled: {}", stats.total_requests);
    println!("  Success rate: {}{:.0}%{}",
        if stats.overall_success_rate() >= 0.8 { colors::OK }
        else if stats.overall_success_rate() >= 0.5 { colors::WARN }
        else { colors::ERR },
        stats.overall_success_rate() * 100.0,
        colors::RESET);
    println!("  Avg reliability: {:.0}", stats.overall_avg_score());

    // Recipes
    let total_recipes = recipe_count();
    if total_recipes > 0 {
        println!("  Learned recipes: {}", total_recipes);
    }

    // Tickets
    if let Ok(ticket_stats) = TicketTracker::for_user().stats() {
        if ticket_stats.total_tickets > 0 {
            println!("  Tickets: {} total, {} resolved",
                ticket_stats.total_tickets, ticket_stats.resolved_tickets);
        }
    }

    // === TEAMS ===
    let active_teams: Vec<_> = stats.by_team.iter()
        .filter(|ts| ts.tickets_total > 0)
        .collect();

    if !active_teams.is_empty() {
        println!();
        println!("{}Teams{}", colors::BOLD, colors::RESET);
        println!("  {:12} {:>6} {:>6} {:>8}", "Team", "Cases", "OK", "Score");
        println!("  {}", "-".repeat(36));

        for ts in active_teams.iter().take(6) {
            let color = if ts.success_rate() >= 0.8 { colors::OK }
                else if ts.success_rate() >= 0.5 { colors::WARN }
                else { colors::ERR };
            println!("  {:12} {:>6} {}{:>6}{} {:>7.0}",
                ts.team,
                ts.tickets_total,
                color, ts.tickets_verified, colors::RESET,
                ts.avg_reliability_score);
        }
    }

    // === TOP PERFORMERS ===
    print_top_performers();

    // === ACHIEVEMENTS ===
    if let Some(ref agg) = agg {
        print_achievements(agg);
    }

    // === FUN FACTS ===
    if let Some(ref agg) = agg {
        print_fun_facts(agg);
    }

    println!();
    println!("{}", HR);
}

/// Print profile/level section
fn print_profile(agg: &AggregatedEvents) {
    // Progress bar
    let xp_for_next = xp_for_level(agg.level + 1);
    let xp_at_start = xp_for_level(agg.level);
    let progress = if xp_for_next > xp_at_start {
        ((agg.xp.saturating_sub(xp_at_start)) as f32 / (xp_for_next - xp_at_start) as f32 * 100.0) as u8
    } else {
        100
    };

    let bar_width = 20;
    let filled = (progress as usize * bar_width) / 100;
    let bar = format!("[{}{}{}{}]",
        colors::OK, "█".repeat(filled),
        colors::DIM, "░".repeat(bar_width - filled));

    println!();
    println!("  {}Level {}{} {}", colors::BOLD, agg.level, colors::RESET, bar);
    println!("  {}{}{}", colors::CYAN, agg.title, colors::RESET);
    println!("  {}XP: {}/{}{}", colors::DIM, agg.xp, xp_for_next, colors::RESET);
}

/// Print top performers
fn print_top_performers() {
    let stats = StaffStats::load();
    if stats.total_tickets() == 0 {
        return;
    }

    let top = stats.top_performers(3);
    if top.is_empty() {
        return;
    }

    println!();
    println!("{}Top Performers{}", colors::BOLD, colors::RESET);

    for (i, (person_id, metrics)) in top.iter().enumerate() {
        let name = person_id.split('_').last()
            .map(|s| {
                let mut c = s.chars();
                c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
            })
            .unwrap_or_else(|| person_id.to_string());

        let medal = match i {
            0 => "[1]",
            1 => "[2]",
            2 => "[3]",
            _ => "   ",
        };

        println!("  {} {} ({} cases, {:.0}% success)",
            medal, name, metrics.tickets_handled, metrics.success_rate());
    }
}

/// Print achievements
fn print_achievements(agg: &AggregatedEvents) {
    let achievements = check_achievements(agg);
    let unlocked: Vec<_> = achievements.iter().filter(|a| a.unlocked).collect();

    if unlocked.is_empty() {
        return;
    }

    println!();
    println!("{}Achievements{}", colors::BOLD, colors::RESET);
    println!("  {}", format_achievements(&achievements, 12));
}

/// Print fun facts
fn print_fun_facts(agg: &AggregatedEvents) {
    if agg.total_requests < 5 {
        return;
    }

    println!();
    println!("{}Highlights{}", colors::BOLD, colors::RESET);

    // Installation date
    if agg.first_event_ts > 0 {
        println!("  Anna since: {} ({})", format_date(agg.first_event_ts), format_tenure(agg.first_event_ts));
    }

    // Streak
    if agg.current_streak > 1 {
        println!("  Current streak: {} days", agg.current_streak);
    }

    // Best team
    if let Some((team, _)) = agg.by_team.iter().max_by_key(|(_, c)| *c) {
        println!("  Favorite team: {}", team);
    }

    // Fastest
    if agg.min_duration_ms < u64::MAX && agg.min_duration_ms > 0 {
        println!("  Fastest answer: {}ms", agg.min_duration_ms);
    }
}

fn xp_for_level(level: u32) -> u64 {
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
