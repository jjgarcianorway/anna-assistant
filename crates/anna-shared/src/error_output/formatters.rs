//! User-friendly output formatters (v0.0.407).
//!
//! v0.0.411: Graceful failure messages with evidence and ticket info
//!
//! Provides simple, honest messages for failed tickets.
//! No LLM terms, no percentages, just clear explanations.

use crate::ticket_state::LiveTicket;

/// v0.0.411: Format failure message with evidence gathered
/// Shows what evidence was collected even when the analysis failed
pub fn format_failure_with_evidence(ticket: &LiveTicket, probes_gathered: &[String]) -> String {
    let mut output = String::new();

    output.push_str("Anna: I tried to process this with my internal IT team but something went wrong in my reasoning.\n");
    output.push_str("I am not confident enough to answer safely.\n");

    // Show gathered evidence
    if !probes_gathered.is_empty() {
        output.push_str("\nEvidence I did gather:\n");
        for probe in probes_gathered.iter().take(5) {
            output.push_str(&format!("  - {}\n", truncate_evidence(probe, 60)));
        }
    }

    // Suggest next steps
    output.push_str("\nYou can:\n");
    output.push_str("  - Run these commands manually to check your system\n");
    output.push_str("  - Ask me again in simpler terms\n");
    output.push_str("  - Try a more specific question\n");

    // Add ticket info for debugging
    output.push_str(&format!("\n(Ticket: {} - {})\n", ticket.id, ticket.domain));

    output
}

/// v0.0.411: Format partial answer with explicit disclaimer
pub fn format_partial_answer(
    answer: &str,
    confident_parts: &str,
    uncertain_parts: &str,
    ticket_id: &str,
    handler: &str,
    evidence: &[String],
) -> String {
    let mut output = answer.to_string();

    // Add explicit disclaimer about partial confidence
    output.push_str("\n\nNote: This is a partial answer. ");
    if !confident_parts.is_empty() {
        output.push_str(&format!("I am confident about {}. ", confident_parts));
    }
    if !uncertain_parts.is_empty() {
        output.push_str(&format!("I am not sure about {}.", uncertain_parts));
    }

    // Add evidence if available
    let evidence_to_show: Vec<_> = evidence.iter().take(3).collect();
    if !evidence_to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for item in evidence_to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(item, 80)));
        }
    }

    // Add ticket and handler info
    output.push_str(&format!(
        "\n\nTicket: {}  handled by {}",
        ticket_id, handler
    ));

    output
}

/// v0.0.411: Format successful answer with ticket info (PART D requirement)
pub fn format_success_with_ticket(
    answer: &str,
    ticket_id: &str,
    handler: &str,
    evidence: &[String],
) -> String {
    let mut output = answer.to_string();

    // Add evidence if available
    let evidence_to_show: Vec<_> = evidence.iter().take(3).collect();
    if !evidence_to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for item in evidence_to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(item, 80)));
        }
    }

    // Add ticket and handler info
    output.push_str(&format!(
        "\n\nTicket: {}  handled by {}",
        ticket_id, handler
    ));

    output
}

/// v0.0.411: Format "missing evidence" response with retry options
pub fn format_missing_evidence(ticket: &LiveTicket, missing_probes: &[String]) -> String {
    let mut output = String::new();

    output.push_str("Anna: I need more information to answer safely.\n");

    if !missing_probes.is_empty() {
        output.push_str("\nI would need:\n");
        for probe in missing_probes.iter().take(4) {
            output.push_str(&format!("  - {}\n", probe));
        }
    }

    output.push_str("\nYou can:\n");
    output.push_str("  - Run these commands and paste the output\n");
    output.push_str("  - Ask me to retry with more probes\n");
    output.push_str("  - Rephrase with more detail about your setup\n");

    output.push_str(&format!("\n(Ticket: {})\n", ticket.id));

    output
}

/// Format a successful answer with evidence (simplified)
///
/// Guidelines:
/// - Max 4-6 lines of main answer
/// - Optional "Evidence" section with 1-3 bullets
/// - No internal LLM terms
/// - No percent success meta statements
pub fn format_success_answer(answer: &str, evidence: &[String], max_evidence: usize) -> String {
    let mut output = answer.to_string();

    // Add evidence if available
    let evidence_to_show: Vec<_> = evidence.iter().take(max_evidence.min(3)).collect();
    if !evidence_to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for item in evidence_to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(item, 80)));
        }
    }

    output
}

/// v0.0.408: Format answer with knowledge item evidence
pub fn format_answer_with_knowledge(
    answer: &str,
    knowledge_titles: &[String],
    max_evidence: usize,
) -> String {
    let mut output = answer.to_string();

    let to_show: Vec<_> = knowledge_titles.iter().take(max_evidence.min(4)).collect();
    if !to_show.is_empty() {
        output.push_str("\n\nEvidence:");
        for title in to_show {
            output.push_str(&format!("\n  - {}", truncate_evidence(title, 80)));
        }
    }

    output
}

/// v0.0.408: Format a "cannot answer" response with suggestions
pub fn format_no_evidence_response(reason: &str, suggestions: &[String]) -> String {
    let mut output = String::from("I cannot safely answer this from local data.");

    if !reason.is_empty() {
        output.push_str(&format!("\n\n{}", reason));
    }

    if !suggestions.is_empty() {
        output.push_str("\n\nYou can try:");
        for suggestion in suggestions.iter().take(5) {
            output.push_str(&format!("\n  - {}", suggestion));
        }
    }

    output
}

/// v0.0.408: Format knowledge search summary for debug
pub fn format_knowledge_debug(
    keywords: &[String],
    found_count: usize,
    source_types: &[String],
) -> String {
    format!(
        "Knowledge search: {} keywords, {} items found from [{}]",
        keywords.len(),
        found_count,
        source_types.join(", ")
    )
}

/// Truncate evidence item
fn truncate_evidence(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
