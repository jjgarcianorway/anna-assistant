//! Response renderer (v0.0.419).
//!
//! This module takes the structured JSON from specialists and renders it
//! into user-facing output with personality. The LLM NEVER generates the
//! personality text - it only provides the data.
//!
//! Key principle: personality is a rendering layer, not LLM output.
//! v0.0.405: Expanded staff for all domains per clean architecture roadmap.

use anna_shared::specialist_contract::SpecialistResponse;

mod builders;
mod formatter;
mod staff;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use formatter::format_for_display;
pub use staff::{get_staff, STAFF};
pub use types::{RenderedResponse, StaffMember};

/// Render a specialist response into user-facing output
pub fn render_response(response: &SpecialistResponse, domain: &str) -> RenderedResponse {
    let staff = get_staff(domain);

    // Build the main answer
    let answer = builders::build_answer(&response.answer.short, &response.answer.detail);

    // Build evidence lines
    let evidence_lines = builders::build_evidence_lines(&response.evidence);

    // Build internal comms (staff personality)
    let internal_comms =
        builders::build_internal_comms(staff, &response.staff_view, &response.status);

    // Calculate reliability
    let reliability = (response.confidence * 100.0) as u8;

    // Status message
    let status_message = builders::build_status_message(&response.status, reliability);

    // v0.0.419: Build citation lines
    let citation_lines = builders::build_citation_lines(&response.citations);

    RenderedResponse {
        answer,
        evidence_lines,
        internal_comms,
        reliability,
        status_message,
        citation_lines,
    }
}
