//! Response renderer (v0.0.404).
//!
//! This module takes the structured JSON from specialists and renders it
//! into user-facing output with personality. The LLM NEVER generates the
//! personality text - it only provides the data.
//!
//! Key principle: personality is a rendering layer, not LLM output.

use anna_shared::specialist_contract::{
    Evidence, Mood, ResponseStatus, Severity, SpecialistResponse, StaffView,
};

/// Staff member definition
#[derive(Debug, Clone)]
pub struct StaffMember {
    pub name: &'static str,
    pub title: &'static str,
}

/// The staff registry
pub const STAFF: &[(&str, StaffMember)] = &[
    (
        "desktop",
        StaffMember {
            name: "Sofia",
            title: "Desktop Administrator",
        },
    ),
    (
        "storage",
        StaffMember {
            name: "Lars",
            title: "Storage Engineer",
        },
    ),
    (
        "system",
        StaffMember {
            name: "Tomas",
            title: "Support Analyst",
        },
    ),
    (
        "network",
        StaffMember {
            name: "Michael",
            title: "Network Engineer",
        },
    ),
    (
        "security",
        StaffMember {
            name: "Elena",
            title: "Security Analyst",
        },
    ),
    (
        "packages",
        StaffMember {
            name: "Marcus",
            title: "Package Manager",
        },
    ),
];

/// Get staff member by domain
pub fn get_staff(domain: &str) -> &'static StaffMember {
    STAFF
        .iter()
        .find(|(d, _)| *d == domain)
        .map(|(_, s)| s)
        .unwrap_or(&StaffMember {
            name: "Anna",
            title: "System Assistant",
        })
}

/// Rendered output for display
#[derive(Debug, Clone)]
pub struct RenderedResponse {
    /// Main answer for the user
    pub answer: String,
    /// Evidence summary lines
    pub evidence_lines: Vec<String>,
    /// Internal comms line (staff chatter)
    pub internal_comms: String,
    /// Reliability percentage
    pub reliability: u8,
    /// Status message
    pub status_message: String,
}

/// Render a specialist response into user-facing output
pub fn render_response(response: &SpecialistResponse, domain: &str) -> RenderedResponse {
    let staff = get_staff(domain);

    // Build the main answer
    let answer = build_answer(&response.answer.short, &response.answer.detail);

    // Build evidence lines
    let evidence_lines = build_evidence_lines(&response.evidence);

    // Build internal comms (staff personality)
    let internal_comms = build_internal_comms(staff, &response.staff_view, &response.status);

    // Calculate reliability
    let reliability = (response.confidence * 100.0) as u8;

    // Status message
    let status_message = build_status_message(&response.status, reliability);

    RenderedResponse {
        answer,
        evidence_lines,
        internal_comms,
        reliability,
        status_message,
    }
}

/// Build the main answer text
fn build_answer(short: &str, detail: &Option<String>) -> String {
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
fn build_evidence_lines(evidence: &[Evidence]) -> Vec<String> {
    evidence
        .iter()
        .map(|e| {
            let snippet = truncate_snippet(&e.snippet, 60);
            format!("- {}: {}", e.probe, snippet)
        })
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
fn build_internal_comms(
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
fn build_status_message(status: &ResponseStatus, reliability: u8) -> String {
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
        ResponseStatus::Error => "System Status | An error occurred processing this request.".to_string(),
    }
}

/// Format the complete output for annactl
pub fn format_for_display(rendered: &RenderedResponse, ticket_id: &str) -> String {
    let mut output = String::new();

    // Internal comms header
    output.push_str("--- internal comms ---\n");
    output.push_str(&format!("  [0.1s] {}\n", rendered.internal_comms));
    output.push('\n');

    // Main answer
    output.push_str("[anna]\n");
    output.push_str(&rendered.answer);
    output.push('\n');

    // Evidence
    if !rendered.evidence_lines.is_empty() {
        output.push('\n');
        for line in &rendered.evidence_lines {
            output.push_str(&format!("  {}\n", line));
        }
    }

    // Footer
    output.push('\n');
    output.push_str(&format!("  ticket: {}\n", ticket_id));
    output.push_str(&rendered.status_message);
    output.push('\n');

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::specialist_contract::Answer;

    #[test]
    fn test_render_ok_response() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0101".to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: "No, there is no active swap configured.".to_string(),
                detail: Some("Both free -h and /proc/swaps show 0B swap.".to_string()),
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: "swap_files".to_string(),
                snippet: "Filename Type Size Used Priority".to_string(),
                interpretation: "No entries listed.".to_string(),
            }],
            confidence: 0.95,
            staff_view: Some(StaffView {
                assignee_role: "System Specialist".to_string(),
                severity: Severity::Info,
                mood: Mood::Confident,
                short_note: Some("No swap configured.".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
        };

        let rendered = render_response(&response, "system");

        assert!(rendered.answer.contains("No, there is no active swap"));
        assert_eq!(rendered.reliability, 95);
        assert!(rendered.internal_comms.contains("Tomas"));
        assert!(rendered.internal_comms.contains("No swap configured"));
    }

    #[test]
    fn test_render_needs_more_data() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0102".to_string(),
            status: ResponseStatus::NeedsMoreData,
            answer: Answer {
                short: "I cannot determine if zram is enabled.".to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.3,
            staff_view: Some(StaffView {
                assignee_role: "System Specialist".to_string(),
                severity: Severity::Unknown,
                mood: Mood::Blocked,
                short_note: Some("Need zram probes.".to_string()),
                complexity: 2,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec!["zram_devices".to_string()],
        };

        let rendered = render_response(&response, "system");

        assert!(rendered.answer.contains("cannot determine"));
        assert!(rendered.internal_comms.contains("Need more data"));
    }

    #[test]
    fn test_format_for_display() {
        let rendered = RenderedResponse {
            answer: "Test answer.".to_string(),
            evidence_lines: vec!["- probe1: snippet1".to_string()],
            internal_comms: "Tomas (Support Analyst): Looks good.".to_string(),
            reliability: 90,
            status_message: "System Status | Verified | 90%".to_string(),
        };

        let output = format_for_display(&rendered, "DSK-0101");

        assert!(output.contains("--- internal comms ---"));
        assert!(output.contains("[anna]"));
        assert!(output.contains("Test answer"));
        assert!(output.contains("ticket: DSK-0101"));
    }
}
