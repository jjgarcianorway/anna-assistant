//! Detailed status sections: team, tickets, health, stats.
//! v0.3.56: Phase 23 - Truthful telemetry from outcome ledger.

use anna_shared::config::AnnaConfig;
use anna_shared::outcome_ledger::OutcomeStats;
use anna_shared::status::DaemonStatus;

use super::colors::*;
use super::formatting::*;

/// Print brief stats from outcome ledger (Phase 23: truthful telemetry).
pub fn print_stats_brief(_status: &DaemonStatus) {
    println_colored("STATS", CYAN);

    // Load from outcome ledger
    let stats = match OutcomeStats::load() {
        Ok(s) if s.total > 0 => s,
        _ => {
            println_colored("  No telemetry data yet.", DIM);
            println!();
            return;
        }
    };

    print!("  requests:      ");
    print!("{}", stats.total);
    if let Some(rate) = stats.success_rate() {
        let color = if rate >= 90.0 { GREEN } else if rate >= 70.0 { YELLOW } else { RED };
        print_colored(&format!(" ({:.0}% success)", rate), color);
    }
    println!();

    if let Some(avg) = stats.avg_duration_ms() {
        print!("  avg time:      ");
        let color = if avg < 1000 { GREEN } else if avg < 5000 { YELLOW } else { DIM };
        println_colored(&format!("{}ms", avg), color);
    }

    if stats.escalated > 0 {
        print!("  escalated:     ");
        println_colored(&format!("{}", stats.escalated), YELLOW);
    }
    println!();
}

pub fn print_team(status: &DaemonStatus) {
    let roster = &status.team_roster;
    if roster.total_specialists == 0 { return; }

    println_colored("TEAM", CYAN);
    print!("  specialists:   ");
    print_colored(&format!("{}", roster.total_specialists), GREEN);
    println_colored(&format!(" across {} departments", roster.specialists.len()), DIM);

    for (dept, specialists) in &roster.specialists {
        let junior_count = specialists.iter().filter(|s| !s.is_senior).count();
        let senior_count = specialists.iter().filter(|s| s.is_senior).count();
        print!("    ");
        print_colored(&format!("{:12}", dept), DIM);
        print!(" ");
        if junior_count > 0 { print_colored(&format!("{}J", junior_count), CYAN); }
        if senior_count > 0 { print!(" "); print_colored(&format!("{}Sr", senior_count), YELLOW); }
        println!();
    }
    println!();
}

pub fn print_tickets(status: &DaemonStatus) {
    let tickets = &status.ticket_tracker;
    if tickets.next_number <= 1 && tickets.active_tickets.is_empty() { return; }

    println_colored("TICKETS", CYAN);
    print!("  today:         ");
    println!("{} tickets", tickets.today_count);

    for (dept, stats) in &tickets.dept_stats {
        if stats.total_received > 0 {
            print!("    ");
            print_colored(&format!("{:12}", dept), DIM);
            print!(" {} handled", stats.total_received);
            if stats.resolved > 0 {
                let rate = stats.resolved as f64 / stats.total_received as f64 * 100.0;
                print_colored(&format!(" ({:.0}% resolved)", rate),
                    if rate >= 80.0 { GREEN } else if rate >= 50.0 { YELLOW } else { RED });
            }
            println!();
        }
    }

    if !tickets.active_tickets.is_empty() {
        println!();
        println_colored("  active:", YELLOW);
        for ticket in &tickets.active_tickets {
            print!("    ");
            print_colored(&ticket.id, CYAN);
            print!(" ");
            let status_color = match ticket.status {
                anna_shared::status::TicketStatus::Open => DIM,
                anna_shared::status::TicketStatus::Investigating => CYAN,
                anna_shared::status::TicketStatus::Experimenting => MAGENTA,
                anna_shared::status::TicketStatus::InProgress => YELLOW,
                anna_shared::status::TicketStatus::Escalated => RED,
                anna_shared::status::TicketStatus::Resolved => GREEN,
                anna_shared::status::TicketStatus::Failed => RED,
            };
            print_colored(&format!("[{}]", ticket.status), status_color);
            print!(" ");
            if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&ticket.created_at) {
                let elapsed = (chrono::Utc::now() - created.with_timezone(&chrono::Utc)).num_seconds();
                if elapsed >= 0 {
                    print_colored(&format!("({})", format_duration(elapsed as u64)), DIM);
                    print!(" ");
                }
            }
            print_colored(&ticket.summary, DIM);
            println!();
        }
    }
    println!();
}

pub fn print_health(status: &DaemonStatus) {
    let has_socket_issues = status.socket_health.status != anna_shared::status::SocketStatus::Healthy
        && status.socket_health.status != anna_shared::status::SocketStatus::Unknown;
    let has_errors = status.error_summary.error_count > 0;

    if !has_socket_issues && !has_errors { return; }

    println_colored("HEALTH", CYAN);
    if has_socket_issues {
        print!("  socket:        ");
        println_colored(&status.socket_health.status.to_string().to_lowercase(), YELLOW);
        if let Some(ref err) = status.socket_health.last_error {
            print_colored("    ", DIM);
            println_colored(err, YELLOW);
        }
    }

    if has_errors {
        print!("  errors:        ");
        println_colored(&format!("{} errors, {} warnings",
            status.error_summary.error_count, status.error_summary.warning_count),
            if status.error_summary.error_count > 0 { RED } else { YELLOW });
        for err in status.error_summary.recent_errors.iter().take(3) {
            print_colored("    [X] ", RED);
            println!("{}", err.message);
        }
    }
    println!();
}

pub fn print_models(status: &DaemonStatus, debug_mode: bool) {
    if status.model_mappings.is_empty() { return; }

    // Only show model mappings in debug mode unless roles differ
    let all_same = status.model_mappings.iter()
        .all(|m| m.model == status.config_snapshot.ollama_model);
    if !debug_mode && all_same { return; }

    println_colored("MODELS", CYAN);
    for mapping in &status.model_mappings {
        print!("  {:14} ", format!("{}:", mapping.role));
        if mapping.is_default {
            println_colored(&mapping.model, DIM);
        } else {
            println_colored(&mapping.model, CYAN);
        }
    }
    println!();
}

pub fn print_config(config: &Option<AnnaConfig>, debug_mode: bool, status: &DaemonStatus) {
    println_colored("CONFIG", CYAN);
    print!("  debug mode:    ");
    if debug_mode {
        println_colored("on", YELLOW);
    } else {
        println_colored("off", GREEN);
    }

    if let Some(ref cfg) = config {
        print!("  auto helpers:  ");
        println_colored(if cfg.auto_install_helpers { "on" } else { "off" },
            if cfg.auto_install_helpers { GREEN } else { DIM });
    }

    if !status.config_snapshot.ollama_model.is_empty() {
        print!("  model:         ");
        println_colored(&status.config_snapshot.ollama_model, DIM);
    }
    println!();
}

/// v0.3.36: Print self-healing recovery status
pub fn print_recovery(status: &DaemonStatus) {
    use anna_shared::status::SubsystemHealth;

    let recovery = &status.recovery_status;
    let overall = recovery.overall_health();

    // Skip if everything is healthy with no activity
    if overall == SubsystemHealth::Healthy && recovery.total_auto_heals == 0 {
        return;
    }

    println_colored("RECOVERY", CYAN);

    print!("  health:        ");
    let health_color = match overall {
        SubsystemHealth::Healthy => GREEN,
        SubsystemHealth::Degraded => YELLOW,
        SubsystemHealth::Recovering => YELLOW,
        SubsystemHealth::Unavailable => RED,
    };
    println_colored(&overall.to_string(), health_color);

    // Show per-subsystem status if not all healthy
    let subsystems = [
        ("ollama", &recovery.ollama),
        ("models", &recovery.models),
        ("wiki", &recovery.wiki),
    ];

    for (name, metrics) in &subsystems {
        if metrics.health != SubsystemHealth::Healthy || metrics.total_attempts > 0 {
            print!("  {:13}  ", name);
            let color = match metrics.health {
                SubsystemHealth::Healthy => GREEN,
                SubsystemHealth::Degraded => YELLOW,
                SubsystemHealth::Recovering => YELLOW,
                SubsystemHealth::Unavailable => RED,
            };
            print_colored(&metrics.health.to_string(), color);
            if metrics.total_attempts > 0 {
                print_colored(
                    &format!(" ({}/{} ok)", metrics.successful_recoveries, metrics.total_attempts),
                    DIM,
                );
            }
            println!();
        }
    }

    if recovery.total_auto_heals > 0 {
        print!("  auto-heals:    ");
        println_colored(&format!("{} total", recovery.total_auto_heals), DIM);
    }

    if let Some(ref last) = recovery.ollama.last_recovery {
        print!("  last recovery: ");
        println_colored(&format_time_ago(last), DIM);
    }

    println!();
}
