//! Statistics section for status display (v0.0.274).
//!
//! Shows department and staff performance metrics in annactl status.
//! v0.0.278: Enhanced RPG stats with XP and title display.

use anna_shared::event_log::EventLog;
use anna_shared::staff_stats::StaffStats;
use anna_shared::ui::colors;
use std::collections::HashMap;

const KEY_WIDTH: usize = 22;

/// Print the statistics section if there are any cases
pub fn print_statistics_section() {
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    let agg = match event_log.aggregate() {
        Ok(a) if a.total_requests > 0 => a,
        _ => return, // No stats to show
    };

    println!();
    println!("{}[statistics]{}", colors::HEADER, colors::RESET);

    // v0.0.278: Show XP and title prominently (RPG gamification)
    kv("xp", &format!("{}{}{}", colors::CYAN, agg.xp, colors::RESET));
    kv("title", &format!("{}{}{}", colors::OK, agg.title, colors::RESET));

    let success_rate = (agg.verified_count as f32 / agg.total_requests as f32) * 100.0;
    let rate_color = if success_rate >= 80.0 { colors::OK }
        else if success_rate >= 50.0 { colors::WARN }
        else { colors::ERR };

    kv("total_cases", &format!("{}", agg.total_requests));
    kv("success_rate", &format!("{}{:.0}%{}", rate_color, success_rate, colors::RESET));
    kv("avg_response", &format!("{:.0}ms", agg.avg_duration_ms));

    // Load staff stats for department and staff breakdown
    let staff_stats = StaffStats::load();
    if staff_stats.total_tickets() == 0 {
        return;
    }

    // Group by team (extract from person_id like "desktop_jr_sofia" -> "desktop")
    let team_stats = aggregate_by_team(&staff_stats);
    print_top_departments(&team_stats);
    print_top_staff(&staff_stats);
}

/// Aggregate staff stats by team/department
fn aggregate_by_team(staff_stats: &StaffStats) -> HashMap<String, (u32, u32, f32)> {
    let mut team_stats: HashMap<String, (u32, u32, f32)> = HashMap::new();

    for (person_id, metrics) in &staff_stats.by_staff {
        let team = person_id.split('_').next().unwrap_or("unknown").to_string();
        let entry = team_stats.entry(team).or_insert((0, 0, 0.0));
        entry.0 += metrics.tickets_handled;
        entry.1 += metrics.tickets_resolved;
        // Running average for reliability
        if entry.0 > 0 {
            entry.2 = (entry.2 * (entry.0 - metrics.tickets_handled) as f32
                + metrics.avg_reliability * metrics.tickets_handled as f32) / entry.0 as f32;
        }
    }

    team_stats
}

/// Print top departments by case count
fn print_top_departments(team_stats: &HashMap<String, (u32, u32, f32)>) {
    if team_stats.is_empty() {
        return;
    }

    let mut teams: Vec<_> = team_stats.iter().collect();
    teams.sort_by(|a, b| b.1.0.cmp(&a.1.0));

    kv("top_departments", "");
    for (team, (handled, resolved, reliability)) in teams.iter().take(3) {
        let rate = if *handled > 0 { (*resolved as f32 / *handled as f32) * 100.0 } else { 0.0 };
        let color = if rate >= 80.0 { colors::OK } else if rate >= 50.0 { colors::WARN } else { colors::ERR };
        println!(
            "    {:12}  cases: {:>3}  ok: {}{:>3}{}  score: {:>5.0}",
            team, handled, color, resolved, colors::RESET, reliability
        );
    }
}

/// Print top staff performers
fn print_top_staff(staff_stats: &StaffStats) {
    let top = staff_stats.top_performers(3);
    if top.is_empty() {
        return;
    }

    kv("top_staff", "");
    for (person_id, metrics) in &top {
        let name = extract_name(person_id);
        let color = if metrics.success_rate() >= 80.0 { colors::OK }
            else if metrics.success_rate() >= 50.0 { colors::WARN }
            else { colors::ERR };
        println!(
            "    {:12}  cases: {:>3}  {}success: {:>5.0}%{}",
            name,
            metrics.tickets_handled,
            color,
            metrics.success_rate(),
            colors::RESET
        );
    }
}

/// v0.0.300: Extract display name from person_id
/// "desktop_jr" -> "Desktop Jr", "desktop_jr_sofia" -> "Sofia (Desktop)"
fn extract_name(person_id: &str) -> String {
    let parts: Vec<&str> = person_id.split('_').collect();

    match parts.len() {
        0 => person_id.to_string(),
        1 => capitalize(parts[0]),
        2 => {
            // "desktop_jr" -> "Desktop Jr"
            let dept = capitalize(parts[0]);
            let role = if parts[1] == "jr" {
                "Jr".to_string()
            } else if parts[1] == "sr" {
                "Sr".to_string()
            } else {
                capitalize(parts[1])
            };
            format!("{} {}", dept, role)
        }
        _ => {
            // "desktop_jr_sofia" -> "Sofia (Desktop)"
            let name = capitalize(parts.last().unwrap_or(&""));
            let dept = capitalize(parts[0]);
            format!("{} ({})", name, dept)
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

fn kv(key: &str, value: &str) {
    println!("  {:width$}{}", key, value, width = KEY_WIDTH);
}
