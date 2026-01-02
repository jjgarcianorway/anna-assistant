//! Public types for the response renderer.

/// Staff member definition
#[derive(Debug, Clone)]
pub struct StaffMember {
    pub name: &'static str,
    pub title: &'static str,
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
    /// v0.0.419: Citation lines for sources footer
    pub citation_lines: Vec<String>,
}
