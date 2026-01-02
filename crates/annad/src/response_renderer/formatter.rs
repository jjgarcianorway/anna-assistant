//! Display formatting for rendered responses.
//!
//! This module handles the final formatting of rendered responses for display.

use super::types::RenderedResponse;

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

    // v0.0.419: Citations/Sources
    if !rendered.citation_lines.is_empty() {
        output.push('\n');
        output.push_str("Sources:\n");
        for line in &rendered.citation_lines {
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
