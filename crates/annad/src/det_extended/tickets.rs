//! Ticket and staff answer functions (v0.0.175).
//!
//! Ticket history, staff roster.

use crate::deterministic::DeterministicResult;

/// Answer ticket history query - shows support desk activity summary (v0.0.116: includes inbox)
pub fn answer_ticket_history(route_class: &str) -> DeterministicResult {
    use anna_shared::email::inbox_path;
    use anna_shared::ticket_tracker::TicketTracker;

    let tracker = TicketTracker::for_user();
    let mut answer = String::new();

    // Check for open tickets
    let open_tickets = tracker.open_tickets().unwrap_or_default();
    let recent_tickets = tracker.recent(5).unwrap_or_default();

    // Check inbox
    let inbox = inbox_path();
    let inbox_count = if inbox.exists() {
        std::fs::read_to_string(&inbox)
            .map(|content| {
                content
                    .lines()
                    .filter(|line| {
                        let trimmed = line.trim();
                        !trimmed.is_empty() && !trimmed.starts_with('#')
                    })
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    // Build answer based on what we found
    if !open_tickets.is_empty() {
        answer.push_str(&format!("**Open Tickets ({}):**\n", open_tickets.len()));
        for ticket in open_tickets.iter().take(5) {
            // Show full query, no truncation
            answer.push_str(&format!(
                "- {} ({})\n  {}\n",
                ticket.case_number, ticket.status, ticket.query
            ));
        }
        answer.push('\n');
    }

    if inbox_count > 0 {
        answer.push_str(&format!(
            "**Inbox:** {} pending {}\n",
            inbox_count,
            if inbox_count == 1 { "query" } else { "queries" }
        ));
        answer.push_str("  Location: ~/.anna/inbox\n\n");
    }

    if open_tickets.is_empty() && inbox_count == 0 {
        answer.push_str("No open tickets or pending queries.\n\n");
    }

    // Add recent history if available
    if !recent_tickets.is_empty() && open_tickets.is_empty() {
        answer.push_str("**Recent Tickets:**\n");
        for ticket in recent_tickets.iter().take(3) {
            // Show full query, no truncation
            answer.push_str(&format!(
                "- {} ({})\n  {}\n",
                ticket.case_number, ticket.status, ticket.query
            ));
        }
        answer.push('\n');
    }

    // Add workflow explanation
    answer.push_str("**How it works:**\n");
    answer.push_str("1. Ask me a question (immediate) or drop it in ~/.anna/inbox (async)\n");
    answer.push_str("2. I create a support ticket and assign the right team\n");
    answer.push_str("3. You get a verified answer with reliability score\n\n");
    answer.push_str("To continue a conversation, just ask me about that ticket.");

    DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: open_tickets.len() + inbox_count,
        route_class: route_class.to_string(),
    }
}

/// Answer staff roster query - shows who is on shift
pub fn answer_staff_roster(route_class: &str) -> DeterministicResult {
    use anna_shared::roster::all_persons;

    let all = all_persons();
    let on_shift: Vec<_> = all.iter().filter(|p| p.is_on_shift()).collect();
    let off_shift_count = all.len() - on_shift.len();

    let mut answer = String::from("**IT Department Staff**\n\n");
    answer.push_str(&format!("Currently on shift ({}):\n", on_shift.len()));

    // Group by team for cleaner display
    let mut teams: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for person in &on_shift {
        teams
            .entry(person.team.to_string())
            .or_default()
            .push(person);
    }

    // Sort teams alphabetically
    let mut team_names: Vec<_> = teams.keys().cloned().collect();
    team_names.sort();

    for team_name in team_names {
        if let Some(members) = teams.get(&team_name) {
            answer.push_str(&format!("\n{} Team:\n", team_name));
            for person in members {
                let specs = if person.specializations.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", person.specializations.join(", "))
                };
                answer.push_str(&format!(
                    "  {} ({}){}\n",
                    person.display_name, person.role_title, specs
                ));
            }
        }
    }

    if off_shift_count > 0 {
        answer.push_str(&format!(
            "\n{} staff members are currently off shift.",
            off_shift_count
        ));
    }

    DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: on_shift.len(),
        route_class: route_class.to_string(),
    }
}
