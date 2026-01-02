//! Teams, helpers, and tickets section handlers for status display.

use anna_shared::helpers::InstallSource;
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::ticket_tracker::{TicketStatus, TicketTracker};
use anna_shared::ui::{colors, kv, print_section_header};

/// Print the [helpers] section (v0.0.453: Enhanced per VISION.md)
pub fn print_helpers_section(snapshot: &StatusSnapshot) {
    if snapshot.helpers.total > 0 {
        print_section_header("helpers");
        // v0.0.453: Better formatting per VISION.md
        if snapshot.helpers.anna_installed > 0 {
            kv(
                "by_anna",
                &format!(
                    "{} (removed on uninstall with confirm)",
                    snapshot.helpers.anna_installed
                ),
            );
            for helper in snapshot
                .helpers
                .list
                .iter()
                .filter(|h| h.source == InstallSource::Anna)
                .take(3)
            {
                let avail = if helper.available {
                    format!("{}available{}", colors::OK, colors::RESET)
                } else {
                    format!("{}missing{}", colors::ERR, colors::RESET)
                };
                println!(
                    "    {} <installed by Anna> [{}]",
                    helper.name, avail
                );
            }
        }
        if snapshot.helpers.user_installed > 0 {
            kv(
                "by_user",
                &format!("{} (kept on uninstall)", snapshot.helpers.user_installed),
            );
            for helper in snapshot
                .helpers
                .list
                .iter()
                .filter(|h| h.source == InstallSource::User)
                .take(3)
            {
                let avail = if helper.available {
                    format!("{}available{}", colors::OK, colors::RESET)
                } else {
                    format!("{}missing{}", colors::ERR, colors::RESET)
                };
                println!(
                    "    {} <installed by user> [{}]",
                    helper.name, avail
                );
            }
        }
    }
}

/// Print the [tickets] section
pub fn print_tickets_section() {
    if let Ok(tickets) = TicketTracker::for_user().open_tickets() {
        if !tickets.is_empty() {
            print_section_header("tickets");
            kv("open", &format!("{}", tickets.len()));
            for ticket in tickets.iter().take(3) {
                let status_color = match ticket.status {
                    TicketStatus::Resolved => colors::OK,
                    TicketStatus::PendingUser => colors::WARN,
                    TicketStatus::InProgress => colors::CYAN,
                    _ => colors::DIM,
                };
                // v0.0.303: Show full query - no truncation for better UX
                println!(
                    "    {}  {}  \"{}\"",
                    ticket.case_number, ticket.team, ticket.query
                );
                println!(
                    "                    status: {}{}{}",
                    status_color,
                    ticket.status.to_string().to_uppercase(),
                    colors::RESET
                );
            }
            kv("escalation_gate", "ENABLED (senior approval required)");
        }
    }
}

/// Print the [teams] section (v0.0.454: Dynamic team availability per VISION.md)
pub fn print_teams_section(snapshot: &StatusSnapshot) {
    if snapshot.teams.available_count > 0 {
        print_section_header("teams");
        kv(
            "available",
            &format!(
                "{}{}{} teams",
                colors::OK,
                snapshot.teams.available_count,
                colors::RESET
            ),
        );
        if snapshot.teams.hidden_count > 0 {
            kv(
                "hidden",
                &format!(
                    "{}{}{} (missing hardware)",
                    colors::DIM,
                    snapshot.teams.hidden_count,
                    colors::RESET
                ),
            );
            for hidden in &snapshot.teams.hidden {
                println!(
                    "    {}{}{}: {}",
                    colors::DIM,
                    hidden.name,
                    colors::RESET,
                    hidden.reason
                );
            }
        }
        // Show hardware detection summary
        let hw = &snapshot.teams.hardware;
        let mut hw_parts = vec![];
        if hw.has_audio {
            hw_parts.push("audio");
        }
        if hw.has_network {
            hw_parts.push("network");
        }
        if hw.has_wifi {
            hw_parts.push("wifi");
        }
        if hw.has_battery {
            hw_parts.push("battery");
        }
        if hw.has_bluetooth {
            hw_parts.push("bluetooth");
        }
        if hw.has_gpu {
            hw_parts.push("gpu");
        }
        if !hw_parts.is_empty() {
            kv("detected_hw", &hw_parts.join(", "));
        }
    }
}
