//! Stats display v2 - RPG-style gamified dashboard (v0.0.250).
//!
//! Matches the user's vision of RPG gamification:
//! - Level/XP with progress bar
//! - Case throughput metrics
//! - Quality scores
//! - Team leaderboard
//! - Achievements

use anna_shared::achievements::{check_achievements, Achievement};
use anna_shared::event_log::EventLog;
use anna_shared::learning_progress::compute_learning_progress;
use anna_shared::learning_suggestions::{generate_suggestions, SuggestionCategory};
use anna_shared::maintenance_actions::{generate_maintenance_actions, ActionCategory};
use anna_shared::snapshot::SystemSnapshot;
use anna_shared::recipe_matcher::recipe_count;
use anna_shared::recipe_store::RecipeStore;
use anna_shared::staff_stats::StaffStats;
use anna_shared::stats::GlobalStats;
use anna_shared::system_telemetry::TelemetryStore;
use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::colors;

use crate::time_format::format_tenure;

const HR: &str = "──────────────────────────────────────────────────────────────────────────────";
const KEY_WIDTH: usize = 22;

/// Print the new RPG-style stats display
pub fn print_stats_display_v2(stats: &GlobalStats) {
    // Load event log for profile stats
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    let agg = event_log.aggregate().ok();

    // === HEADER ===
    println!("{}", HR);
    let title = agg.as_ref().map(|a| a.title.as_str()).unwrap_or("IT Newcomer");
    let level = agg.as_ref().map(|a| a.level).unwrap_or(1);
    println!(
        "Anna Service Desk  |  {}{}{}  |  Level {}",
        colors::CYAN, title, colors::RESET, level
    );
    println!("{}", HR);

    // === [profile] ===
    println!();
    println!("{}[profile]{}", colors::HEADER, colors::RESET);

    if let Some(ref agg) = agg {
        kv("title", &format!("{}{}{}", colors::CYAN, agg.title, colors::RESET));
        kv("level", &format!("{}", agg.level));

        // XP progress bar
        let xp_for_next = xp_for_level(agg.level + 1);
        let xp_at_start = xp_for_level(agg.level);
        let xp_in_level = agg.xp.saturating_sub(xp_at_start);
        let xp_needed = xp_for_next.saturating_sub(xp_at_start);
        let progress = if xp_needed > 0 {
            (xp_in_level as f32 / xp_needed as f32 * 100.0) as u8
        } else {
            100
        };

        let bar = make_progress_bar(progress, 20);
        kv("xp", &format!("{} / {}  {}", agg.xp, xp_for_next, bar));
        kv("xp_to_next", &format!("{}", xp_needed.saturating_sub(xp_in_level)));

        if agg.first_event_ts > 0 {
            kv("tenure", &format_tenure(agg.first_event_ts));
        }
        if agg.current_streak > 0 {
            kv("current_streak", &format!("{} days", agg.current_streak));
        }
    } else {
        kv("title", "IT Newcomer");
        kv("level", "1");
        kv("xp", &format!("0 / 100  {}", make_progress_bar(0, 20)));
    }

    // === [throughput] ===
    println!();
    println!("{}[throughput]{}", colors::HEADER, colors::RESET);

    let total_requests = agg.as_ref().map(|a| a.total_requests).unwrap_or(stats.total_requests);
    let verified = agg.as_ref().map(|a| a.verified_count).unwrap_or(0);
    let failed = agg.as_ref().map(|a| a.failed_count).unwrap_or(0);
    let timeouts = agg.as_ref().map(|a| a.timeout_count).unwrap_or(0);

    kv("total_cases", &format!("{}", total_requests));
    kv("resolved_ok", &format!("{}{}{}", colors::OK, verified, colors::RESET));
    kv("failed", &format!("{}{}{}", if failed > 0 { colors::ERR } else { colors::DIM }, failed, colors::RESET));
    kv("timeouts", &format!("{}", timeouts));

    if let Some(ref agg) = agg {
        kv("escalations", &format!("{}", agg.escalation_count));
        kv("clarifications", &format!("{}", agg.clarification_count));
    }

    // === [quality] ===
    println!();
    println!("{}[quality]{}", colors::HEADER, colors::RESET);

    let success_rate = stats.overall_success_rate();
    let success_color = if success_rate >= 0.8 { colors::OK }
        else if success_rate >= 0.5 { colors::WARN }
        else { colors::ERR };
    kv("success_rate", &format!("{}{:.0}%{}", success_color, success_rate * 100.0, colors::RESET));
    kv("avg_reliability", &format!("{:.0}", stats.overall_avg_score()));

    if let Some(ref agg) = agg {
        if agg.avg_duration_ms > 0.0 {
            kv("avg_response_time", &format!("{:.0}ms", agg.avg_duration_ms));
            kv("fastest_ever", &format!("{}ms", agg.min_duration_ms));
            kv("slowest_ever", &format!("{}ms", agg.max_duration_ms));
        }
    }

    kv("fast_path_hits", &format!("{} ({:.0}%)", stats.fast_path_hits, stats.fast_path_percentage()));

    // === [learning] === v0.0.288: Now shows Anna's growth
    println!();
    println!("{}[learning]{}", colors::HEADER, colors::RESET);

    // v0.0.288: Show learning progress (data-driven)
    let progress = compute_learning_progress();

    let total_recipes = recipe_count();
    kv("recipes_learned", &format!("{}", total_recipes));

    // Self-sufficiency shows how much Anna handles on her own
    if stats.total_requests > 0 {
        let self_pct = ((stats.fast_path_hits + stats.recipe_hits) as f32
            / stats.total_requests as f32
            * 100.0) as u8;
        let color = if self_pct >= 50 {
            colors::OK
        } else if self_pct >= 20 {
            colors::WARN
        } else {
            colors::DIM
        };
        kv("self_sufficiency", &format!("{}{}%{}", color, self_pct, colors::RESET));
    }

    // Show strong areas (from actual data)
    if !progress.strong_areas.is_empty() {
        kv("strong_in", &progress.strong_areas.join(", "));
    }

    // Show growing areas
    if !progress.growing_areas.is_empty() && progress.growing_areas.len() <= 3 {
        kv("learning", &progress.growing_areas.join(", "));
    }

    if stats.knowledge_pack_hits > 0 {
        kv("knowledge_pack_hits", &format!("{}", stats.knowledge_pack_hits));
    }
    if stats.recipe_hits > 0 {
        kv("recipe_cache_hits", &format!("{}", stats.recipe_hits));
    }

    // Tickets
    if let Ok(ticket_stats) = TicketTracker::for_user().stats() {
        if ticket_stats.total_tickets > 0 {
            kv("tickets_tracked", &format!("{} total, {} resolved",
                ticket_stats.total_tickets, ticket_stats.resolved_tickets));
        }
    }

    // === [team leaderboard] ===
    let staff_stats = StaffStats::load();
    if staff_stats.total_tickets() > 0 {
        println!();
        println!("{}[team leaderboard]{}", colors::HEADER, colors::RESET);

        let top = staff_stats.top_performers(5);
        for (i, (person_id, metrics)) in top.iter().enumerate() {
            let name = extract_name(person_id);
            let medal = match i {
                0 => format!("{}[1]{}", colors::OK, colors::RESET),
                1 => format!("{}[2]{}", colors::CYAN, colors::RESET),
                2 => format!("{}[3]{}", colors::WARN, colors::RESET),
                _ => format!("{}[{}]{}", colors::DIM, i + 1, colors::RESET),
            };
            println!(
                "    {} {:12}  cases: {:>3}  success: {}{:>5.0}%{}",
                medal,
                name,
                metrics.tickets_handled,
                if metrics.success_rate() >= 80.0 { colors::OK } else { colors::DIM },
                metrics.success_rate(),
                colors::RESET
            );
        }
    }

    // === [teams] ===
    let active_teams: Vec<_> = stats.by_team.iter()
        .filter(|ts| ts.tickets_total > 0)
        .collect();

    if !active_teams.is_empty() {
        println!();
        println!("{}[teams]{}", colors::HEADER, colors::RESET);

        for ts in active_teams.iter().take(6) {
            let color = if ts.success_rate() >= 0.8 { colors::OK }
                else if ts.success_rate() >= 0.5 { colors::WARN }
                else { colors::ERR };
            println!(
                "    {:12}  cases: {:>3}  ok: {}{:>3}{}  score: {:>5.0}",
                ts.team, ts.tickets_total, color, ts.tickets_verified, colors::RESET, ts.avg_reliability_score
            );
        }
    }

    // === [achievements] ===
    if let Some(ref agg) = agg {
        let achievements = check_achievements(agg);
        let unlocked: Vec<_> = achievements.iter().filter(|a| a.unlocked).collect();
        let locked_count = achievements.len() - unlocked.len();

        if !unlocked.is_empty() || locked_count > 0 {
            println!();
            println!("{}[achievements]{}", colors::HEADER, colors::RESET);
            kv("unlocked", &format!("{} / {}", unlocked.len(), achievements.len()));

            // Show unlocked achievements as icons
            if !unlocked.is_empty() {
                let icons: String = unlocked.iter().map(|a| format_achievement_icon(a)).collect::<Vec<_>>().join(" ");
                println!("    {}", icons);
            }

            // Show next achievement to unlock
            if let Some(next) = achievements.iter().find(|a| !a.unlocked) {
                kv("next_unlock", &format!("{}{}{}", colors::DIM, next.name, colors::RESET));
            }
        }
    }

    // === [suggestions] === v0.0.283
    print_suggestions_section();

    // === [maintenance] === v0.0.286
    print_maintenance_section();

    println!("{}", HR);
}

/// Print learning suggestions section
fn print_suggestions_section() {
    // Load recipe store and telemetry if available
    let recipe_store = RecipeStore::load(RecipeStore::default_path()).ok();
    let telemetry = TelemetryStore::load_if_exists();

    let suggestions = generate_suggestions(
        recipe_store.as_ref(),
        telemetry.as_ref(),
    );

    if suggestions.is_empty() {
        return;
    }

    println!();
    println!("{}[suggestions]{}", colors::HEADER, colors::RESET);

    for (i, suggestion) in suggestions.iter().take(3).enumerate() {
        let category_icon = match suggestion.category {
            SuggestionCategory::NewDomain => "[+]",
            SuggestionCategory::DeepDive => "[>]",
            SuggestionCategory::KnowledgeGap => "[?]",
            SuggestionCategory::Improvement => "[^]",
            SuggestionCategory::SystemHealth => "[!]",
        };

        let priority_color = if suggestion.priority <= 2 { colors::WARN } else { colors::DIM };

        println!(
            "  {}. {}{}{} {}",
            i + 1,
            priority_color,
            category_icon,
            colors::RESET,
            suggestion.title
        );

        if let Some(ref example) = suggestion.example_query {
            println!(
                "       {}Try: \"{}\"{}",
                colors::DIM, example, colors::RESET
            );
        }
    }
}

/// v0.0.286: Print maintenance actions section
fn print_maintenance_section() {
    // Get current snapshot and telemetry
    let snapshot = SystemSnapshot::now();
    let telemetry = TelemetryStore::load_if_exists();

    let actions = generate_maintenance_actions(&snapshot, telemetry.as_ref());

    // Only show if there are urgent actions (urgency <= 3)
    let urgent_actions: Vec<_> = actions.iter().filter(|a| a.urgency <= 3).collect();
    if urgent_actions.is_empty() {
        return;
    }

    println!();
    println!("{}[maintenance]{}", colors::HEADER, colors::RESET);

    for (i, action) in urgent_actions.iter().take(3).enumerate() {
        let urgency_marker = match action.urgency {
            1 => format!("{}[!!]{}", colors::ERR, colors::RESET),
            2 => format!("{}[! ]{}", colors::WARN, colors::RESET),
            _ => format!("{}[* ]{}", colors::DIM, colors::RESET),
        };

        let category_hint = match action.category {
            ActionCategory::DiskCleanup => "disk",
            ActionCategory::MemoryOptimize => "memory",
            ActionCategory::ServiceRepair => "service",
            ActionCategory::SecurityAudit => "security",
            ActionCategory::PerformanceTune => "perf",
            ActionCategory::SystemUpdate => "update",
        };

        println!(
            "  {}. {} {}{} {}",
            i + 1,
            urgency_marker,
            action.title,
            format!(" {}({}){}", colors::DIM, category_hint, colors::RESET),
            ""
        );

        println!(
            "       {}Ask: \"{}\"{}",
            colors::DIM, action.anna_query, colors::RESET
        );
    }
}

fn kv(key: &str, value: &str) {
    println!("  {:width$}{}", key, value, width = KEY_WIDTH);
}

fn make_progress_bar(percent: u8, width: usize) -> String {
    let filled = (percent as usize * width) / 100;
    format!(
        "[{}{}{}{}{}]",
        colors::OK,
        "█".repeat(filled),
        colors::DIM,
        "░".repeat(width.saturating_sub(filled)),
        colors::RESET
    )
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

fn extract_name(person_id: &str) -> String {
    person_id
        .split('_')
        .next_back()
        .map(|s| {
            let mut c = s.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .unwrap_or_else(|| person_id.to_string())
}

fn format_achievement_icon(achievement: &Achievement) -> String {
    // v0.0.265: ASCII icons instead of emojis
    let icon = match achievement.id {
        "first_request" => "[1]",
        "ten_requests" => "[10]",
        "hundred_requests" => "[100]",
        "first_verified" => "[v]",
        "fast_responder" => "[*]",
        "no_timeouts" => "[t]",
        "recipe_learner" => "[r]",
        "escalation_master" => "[^]",
        "streak_3" => "[3d]",
        "streak_7" => "[7d]",
        "streak_30" => "[30d]",
        _ => "[+]",
    };
    format!("{}{}{}", icon, colors::DIM, colors::RESET)
}
