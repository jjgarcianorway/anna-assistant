//! Response component builders.
//!
//! This module contains functions to build various components of the rendered response:
//! - Answer text
//! - Evidence lines
//! - Citation lines
//! - Internal comms (staff personality)
//! - Status messages

use anna_shared::specialist_contract::{
    Evidence, KnowledgeCitation, Mood, ResponseStatus, Severity, StaffView,
};

use super::types::StaffMember;

/// Build the main answer text
pub fn build_answer(short: &str, detail: &Option<String>) -> String {
    let mut answer = short.to_string();

    if let Some(detail) = detail {
        if !detail.is_empty() {
            answer.push_str("\n\n");
            answer.push_str(detail);
        }
    }

    answer
}

/// Build evidence summary lines
pub fn build_evidence_lines(evidence: &[Evidence]) -> Vec<String> {
    evidence
        .iter()
        .map(|e| {
            let snippet = truncate_snippet(&e.snippet, 60);
            format!("- {}: {}", e.probe, snippet)
        })
        .collect()
}

/// v0.0.419: Build citation lines for sources footer
pub fn build_citation_lines(citations: &[KnowledgeCitation]) -> Vec<String> {
    citations
        .iter()
        .map(|c| format!("- {}", c.footer_display()))
        .collect()
}

/// Truncate a snippet for display
fn truncate_snippet(snippet: &str, max_len: usize) -> String {
    let clean = snippet.replace('\n', " ").trim().to_string();
    if clean.len() <= max_len {
        clean
    } else {
        format!("{}...", &clean[..max_len])
    }
}

/// Build internal comms (staff personality)
pub fn build_internal_comms(
    staff: &StaffMember,
    staff_view: &Option<StaffView>,
    status: &ResponseStatus,
) -> String {
    let note = staff_view
        .as_ref()
        .and_then(|v| v.short_note.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    let severity = staff_view
        .as_ref()
        .map(|v| &v.severity)
        .unwrap_or(&Severity::Info);

    let mood = staff_view
        .as_ref()
        .map(|v| &v.mood)
        .unwrap_or(&Mood::Confident);

    // Build the staff line using templates
    let prefix = format!("{} ({}):", staff.name, staff.title);

    let message = match (status, severity, mood) {
        (ResponseStatus::Error, _, _) => "Something went wrong. Need to escalate.".to_string(),
        (ResponseStatus::NeedsMoreData, _, _) => {
            format!("Need more data. {}", note)
        }
        (ResponseStatus::CannotAnswer, _, _) => {
            format!("Can't answer this one. {}", note)
        }
        (ResponseStatus::NoEvidence, _, _) => {
            format!("No evidence collected yet. {}", note)
        }
        (ResponseStatus::Ok, Severity::Critical, _) => {
            format!("This is critical: {}", note)
        }
        (ResponseStatus::Ok, Severity::Warning, _) => {
            format!("Heads up: {}", note)
        }
        (ResponseStatus::Ok, Severity::Unknown, Mood::Blocked) => {
            format!("Blocked: {}", note)
        }
        (ResponseStatus::Ok, Severity::Unknown, Mood::Uncertain) => {
            format!("Not sure about this: {}", note)
        }
        (ResponseStatus::Ok, _, Mood::Confident) => {
            if note.is_empty() {
                "Looks good.".to_string()
            } else {
                note.to_string()
            }
        }
        (ResponseStatus::Ok, _, Mood::Uncertain) => {
            format!("Think this is right: {}", note)
        }
        (ResponseStatus::Ok, _, _) => {
            if note.is_empty() {
                "Done.".to_string()
            } else {
                note.to_string()
            }
        }
    };

    format!("{} {}", prefix, message)
}

/// Build status message
pub fn build_status_message(status: &ResponseStatus, reliability: u8) -> String {
    match status {
        ResponseStatus::Ok => {
            format!(
                "System Status | This information is verified from system data. | {}%",
                reliability
            )
        }
        ResponseStatus::NeedsMoreData => {
            "System Status | Need additional probes to answer completely.".to_string()
        }
        ResponseStatus::CannotAnswer => {
            "System Status | Cannot answer this question with available data.".to_string()
        }
        ResponseStatus::Error => {
            "System Status | An error occurred processing this request.".to_string()
        }
        ResponseStatus::NoEvidence => {
            "System Status | No evidence was collected for this query.".to_string()
        }
    }
}
