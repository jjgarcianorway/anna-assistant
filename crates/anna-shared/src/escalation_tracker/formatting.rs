// v0.0.529: Escalation Tracker Formatting (Phase 105)
// Display and formatting functions for escalations

use crate::escalation_tracker::{EscalationRecord, EscalationTracker};

/// Format escalation for display
pub fn format_escalation(esc: &EscalationRecord) -> String {
    format!(
        "{} (Ticket: {})\n  {} → {} [{}]\n  Reason: {} | Outcome: {}\n  Time: {}",
        esc.id,
        esc.ticket_id,
        esc.from_specialist,
        esc.to_specialist,
        esc.department,
        esc.reason,
        esc.outcome,
        if let Some(ms) = esc.resolution_ms {
            format!("{}ms", ms)
        } else {
            "Pending".to_string()
        }
    )
}

/// Format escalation compact
pub fn format_escalation_compact(esc: &EscalationRecord) -> String {
    format!(
        "{}: {} → {} ({})",
        esc.id, esc.from_specialist, esc.to_specialist, esc.reason
    )
}

/// Format escalation oneline
pub fn format_escalation_oneline(esc: &EscalationRecord) -> String {
    format!("{} [{}]", esc.id, esc.outcome)
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &EscalationTracker, total_tickets: u32) -> String {
    let mut output = String::new();
    output.push_str("=== Escalation Summary ===\n\n");

    output.push_str(&format!("Total Escalations: {}\n", tracker.total()));
    output.push_str(&format!(
        "Escalation Rate: {:.1}%\n",
        tracker.escalation_rate(total_tickets)
    ));
    output.push_str(&format!(
        "Senior Resolution Rate: {:.1}%\n",
        tracker.senior_resolution_rate()
    ));

    if let Some(avg) = tracker.avg_resolution_ms() {
        output.push_str(&format!("Avg Resolution Time: {}ms\n", avg));
    }

    output.push_str(&format!("Pending: {}\n\n", tracker.pending().len()));

    output.push_str("--- By Reason ---\n");
    for (reason, count) in tracker.reason_stats() {
        output.push_str(&format!("  {}: {}\n", reason, count));
    }

    output
}
