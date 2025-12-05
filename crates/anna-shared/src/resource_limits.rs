//! Resource limits and diagnostics for COST phase.
//!
//! Explicit caps with surfaced diagnostics - never silent truncation.
//! When a cap is hit, we record what was dropped and the consequence.

use serde::{Deserialize, Serialize};

// === RESOURCE LIMITS (configurable as constants, can move to config later) ===

/// Maximum transcript events per request.
/// Beyond this, new events are dropped with diagnostic.
pub const MAX_TRANSCRIPT_EVENTS: usize = 100;

/// Maximum LLM prompt characters.
/// Probe results are truncated to fit within this budget.
pub const MAX_PROMPT_CHARS: usize = 8000;

/// Maximum probe output chars per probe (before summarization).
/// Individual probes exceeding this are truncated at source.
pub const MAX_PROBE_OUTPUT_CHARS: usize = 2000;

// === RESOURCE DIAGNOSTIC ===

/// What resource was capped
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    /// Transcript events exceeded limit
    TranscriptEvents,
    /// LLM prompt chars exceeded limit
    PromptChars,
    /// Individual probe output exceeded limit
    ProbeOutput,
}

impl ResourceKind {
    /// Short name for logging
    pub fn name(&self) -> &'static str {
        match self {
            Self::TranscriptEvents => "transcript",
            Self::PromptChars => "prompt",
            Self::ProbeOutput => "probe_output",
        }
    }
}

/// Diagnostic for a resource cap hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDiagnostic {
    /// What resource was capped
    pub kind: ResourceKind,
    /// The limit that was hit
    pub limit: usize,
    /// What was dropped (count or bytes)
    pub dropped: usize,
    /// What the operational consequence is
    pub consequence: String,
}

impl ResourceDiagnostic {
    /// Create transcript cap diagnostic
    pub fn transcript_capped(dropped_events: usize) -> Self {
        Self {
            kind: ResourceKind::TranscriptEvents,
            limit: MAX_TRANSCRIPT_EVENTS,
            dropped: dropped_events,
            consequence: "debug output incomplete, reliability penalty applies".to_string(),
        }
    }

    /// Create prompt cap diagnostic
    pub fn prompt_truncated(dropped_chars: usize) -> Self {
        Self {
            kind: ResourceKind::PromptChars,
            limit: MAX_PROMPT_CHARS,
            dropped: dropped_chars,
            consequence: "analysis degraded, reliability penalty applies".to_string(),
        }
    }

    /// Create probe output cap diagnostic
    pub fn probe_truncated(dropped_chars: usize) -> Self {
        Self {
            kind: ResourceKind::ProbeOutput,
            limit: MAX_PROBE_OUTPUT_CHARS,
            dropped: dropped_chars,
            consequence: "probe data incomplete".to_string(),
        }
    }

    /// Format as single-line diagnostic message
    pub fn format(&self) -> String {
        format!(
            "{} cap hit (limit {}, dropped {}): {}",
            self.kind.name(),
            self.limit,
            self.dropped,
            self.consequence
        )
    }
}

/// Collection of resource diagnostics for a request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceDiagnostics {
    /// All diagnostics recorded during request
    pub items: Vec<ResourceDiagnostic>,
}

impl ResourceDiagnostics {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, diagnostic: ResourceDiagnostic) {
        self.items.push(diagnostic);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Check if transcript was capped
    pub fn transcript_capped(&self) -> bool {
        self.items
            .iter()
            .any(|d| d.kind == ResourceKind::TranscriptEvents)
    }

    /// Check if prompt was truncated
    pub fn prompt_truncated(&self) -> bool {
        self.items
            .iter()
            .any(|d| d.kind == ResourceKind::PromptChars)
    }

    /// Get total dropped across all diagnostics
    pub fn total_dropped(&self) -> usize {
        self.items.iter().map(|d| d.dropped).sum()
    }

    /// Format all diagnostics as multi-line string
    pub fn format_all(&self) -> String {
        self.items
            .iter()
            .map(|d| d.format())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_diagnostic() {
        let diag = ResourceDiagnostic::transcript_capped(5);
        assert_eq!(diag.kind, ResourceKind::TranscriptEvents);
        assert_eq!(diag.dropped, 5);
        assert!(diag.format().contains("transcript"));
        assert!(diag.format().contains("reliability penalty"));
    }

    #[test]
    fn test_prompt_diagnostic() {
        let diag = ResourceDiagnostic::prompt_truncated(1500);
        assert_eq!(diag.kind, ResourceKind::PromptChars);
        assert_eq!(diag.dropped, 1500);
        assert!(diag.format().contains("analysis degraded"));
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diags = ResourceDiagnostics::new();
        assert!(diags.is_empty());
        assert!(!diags.transcript_capped());
        assert!(!diags.prompt_truncated());

        diags.push(ResourceDiagnostic::transcript_capped(10));
        assert!(!diags.is_empty());
        assert!(diags.transcript_capped());
        assert!(!diags.prompt_truncated());

        diags.push(ResourceDiagnostic::prompt_truncated(500));
        assert!(diags.prompt_truncated());
        assert_eq!(diags.total_dropped(), 510);
    }

    #[test]
    fn test_format_all() {
        let mut diags = ResourceDiagnostics::new();
        diags.push(ResourceDiagnostic::transcript_capped(5));
        diags.push(ResourceDiagnostic::prompt_truncated(200));

        let formatted = diags.format_all();
        assert!(formatted.contains("transcript"));
        assert!(formatted.contains("prompt"));
    }
}
