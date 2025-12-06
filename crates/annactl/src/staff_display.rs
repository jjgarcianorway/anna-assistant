//! Staff roster and workload display for Service Desk Theatre (v0.0.110).
//!
//! Shows the IT department team with specializations, shifts, and workload stats.

use anna_shared::roster::{all_persons, person_by_id, Tier};
use anna_shared::staff_stats::StaffStats;
use anna_shared::teams::Team;
use anna_shared::ui::colors;

/// Display the full IT department roster
pub fn print_staff_roster() {
    println!();
    println!("{}Anna Service Desk - IT Department{}", colors::BOLD, colors::RESET);
    println!();

    let stats = StaffStats::load();
    let persons = all_persons();

    // Group by team
    let teams = [
        Team::Desktop, Team::Network, Team::Hardware, Team::Storage,
        Team::Performance, Team::Security, Team::Services, Team::Logs, Team::General,
    ];

    for team in teams {
        let team_members: Vec<_> = persons.iter()
            .filter(|p| p.team == team)
            .collect();

        if team_members.is_empty() {
            continue;
        }

        println!("{}{}:{}", colors::CYAN, team, colors::RESET);

        for person in team_members {
            let tier_badge = match person.tier {
                Tier::Junior => format!("{}[jr]{}", colors::DIM, colors::RESET),
                Tier::Senior => format!("{}[sr]{}", colors::WARN, colors::RESET),
            };

            // Shift status
            let shift_indicator = if person.is_on_shift() {
                format!("{}●{}", colors::OK, colors::RESET)
            } else {
                format!("{}○{}", colors::DIM, colors::RESET)
            };

            // Get workload stats for this person
            let workload = stats.get(person.person_id)
                .map(|m| format_workload(m))
                .unwrap_or_else(|| format!("{}idle{}", colors::DIM, colors::RESET));

            println!(
                "  {} {} {} {}",
                shift_indicator, tier_badge, person.display_name, workload
            );

            // Show shift and specializations
            let shift_str = format!("{} shift", person.shift);
            if !person.specializations.is_empty() {
                println!(
                    "      {}{} | Specializes in: {}{}",
                    colors::DIM, shift_str, person.specialization_str(), colors::RESET
                );
            } else {
                println!(
                    "      {}{}{}",
                    colors::DIM, shift_str, colors::RESET
                );
            }
        }
        println!();
    }

    // Summary
    print_department_summary(&stats);
}

/// Format workload indicator
fn format_workload(metrics: &anna_shared::staff_stats::StaffMetrics) -> String {
    if metrics.tickets_handled == 0 {
        return format!("{}idle{}", colors::DIM, colors::RESET);
    }

    let rate = metrics.success_rate();
    let color = if rate >= 80.0 {
        colors::OK
    } else if rate >= 50.0 {
        colors::WARN
    } else {
        colors::ERR
    };

    format!(
        "{}{} tickets, {:.0}% success{}",
        color, metrics.tickets_handled, rate, colors::RESET
    )
}

/// Print department summary
fn print_department_summary(stats: &StaffStats) {
    println!("{}Department Summary{}", colors::BOLD, colors::RESET);

    // Currently on shift
    let on_shift_count = all_persons().iter()
        .filter(|p| p.is_on_shift())
        .count();
    println!("  Currently on shift: {}/18", on_shift_count);

    let total = stats.total_tickets();
    if total == 0 {
        println!("  {}No tickets handled yet.{}", colors::DIM, colors::RESET);
        println!();
        return;
    }

    // Active staff count (have handled tickets)
    let active_count = stats.by_staff.iter()
        .filter(|(_, m)| m.tickets_handled > 0)
        .count();

    println!("  Staff with activity: {}/18", active_count);
    println!("  Total tickets: {}", total);

    // Best performer
    if let Some((id, metrics)) = stats.top_performers(1).first() {
        if let Some(person) = person_by_id(id) {
            println!(
                "  {}Top performer:{} {} ({:.0}% success)",
                colors::OK, colors::RESET,
                person.display_name,
                metrics.success_rate()
            );
        }
    }

    // Busiest team
    let team_tickets: Vec<(Team, u32)> = [
        Team::Desktop, Team::Network, Team::Hardware, Team::Storage,
        Team::Performance, Team::Security, Team::Services, Team::Logs, Team::General,
    ].iter().map(|team| {
        let count: u32 = all_persons().iter()
            .filter(|p| p.team == *team)
            .filter_map(|p| stats.get(p.person_id))
            .map(|m| m.tickets_handled)
            .sum();
        (*team, count)
    }).collect();

    if let Some((team, count)) = team_tickets.iter().max_by_key(|(_, c)| c) {
        if *count > 0 {
            println!("  Busiest team: {} ({} tickets)", team, count);
        }
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_workload_idle() {
        let metrics = anna_shared::staff_stats::StaffMetrics::default();
        let result = format_workload(&metrics);
        assert!(result.contains("idle"));
    }
}
