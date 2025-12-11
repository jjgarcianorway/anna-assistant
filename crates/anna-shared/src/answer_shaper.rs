//! Answer Shaper (v0.0.415).
//!
//! Shapes specialist responses into user-friendly Anna output.
//! Anna is the narrator - she reads specialist JSON and presents it nicely.
//!
//! Key principles:
//! - Short answers for simple queries (1-2 lines)
//! - Evidence footer only when helpful
//! - No verbose IT department chatter for simple questions
//! - Honest failures instead of fake success

use crate::strict_contract::{StrictSpecialistResponse, StrictStatus};
use crate::translator_contract::TranslatorIntent;

/// Shape a specialist response for user display
pub fn shape_answer(
    response: &StrictSpecialistResponse,
    intent: TranslatorIntent,
    show_evidence: bool,
    show_internal: bool,
) -> ShapedAnswer {
    // Determine answer style based on intent and status
    let style = determine_style(intent, &response.status, response.confidence);

    // Build the answer text
    let text = build_answer_text(response, style, show_evidence);

    // Build evidence line
    let evidence_line = if show_evidence && !response.evidence.is_empty() {
        let probes: Vec<&str> = response.evidence.iter().map(|e| e.probe.as_str()).collect();
        Some(format!("Evidence: {}", probes.join(", ")))
    } else {
        None
    };

    ShapedAnswer {
        text,
        evidence_line,
        is_resolved: response.is_resolved(),
        confidence: response.confidence,
        status: match response.status {
            StrictStatus::Ok => "resolved",
            StrictStatus::Partial => "partial",
            StrictStatus::Failed => "failed",
        },
        internal_note: if show_internal {
            Some(format!(
                "[{}] confidence={:.0}%, evidence={}",
                response.status_str(),
                response.confidence * 100.0,
                response.evidence.len()
            ))
        } else {
            None
        },
    }
}

/// Shaped answer for display
#[derive(Debug, Clone)]
pub struct ShapedAnswer {
    /// Main answer text
    pub text: String,
    /// Optional evidence line
    pub evidence_line: Option<String>,
    /// Whether this counts as resolved
    pub is_resolved: bool,
    /// Confidence score
    pub confidence: f32,
    /// Status string
    pub status: &'static str,
    /// Internal note (if enabled)
    pub internal_note: Option<String>,
}

impl ShapedAnswer {
    /// Format for display
    pub fn format(&self) -> String {
        let mut output = self.text.clone();

        if let Some(evidence) = &self.evidence_line {
            output.push_str("\n\n");
            output.push_str(evidence);
        }

        output
    }

    /// Format with internal note
    pub fn format_with_internal(&self) -> String {
        let mut output = self.format();

        if let Some(note) = &self.internal_note {
            output.push_str("\n\n");
            output.push_str(note);
        }

        output
    }
}

/// Answer style based on context
#[derive(Debug, Clone, Copy)]
enum AnswerStyle {
    /// Simple one-liner (e.g., "how much RAM?")
    Concise,
    /// Short answer with optional bullets (e.g., "why is boot slow?")
    Diagnostic,
    /// Failed/partial - honest about limitations
    Honest,
}

fn determine_style(intent: TranslatorIntent, status: &StrictStatus, confidence: f32) -> AnswerStyle {
    if *status == StrictStatus::Failed || confidence < 0.5 {
        return AnswerStyle::Honest;
    }

    match intent {
        TranslatorIntent::QueryMetric | TranslatorIntent::CheckStatus => AnswerStyle::Concise,
        TranslatorIntent::Diagnose | TranslatorIntent::List => AnswerStyle::Diagnostic,
        TranslatorIntent::Configure | TranslatorIntent::Explain => AnswerStyle::Diagnostic,
    }
}

fn build_answer_text(response: &StrictSpecialistResponse, style: AnswerStyle, _show_evidence: bool) -> String {
    match style {
        AnswerStyle::Concise => {
            // Just the summary, clean and simple
            response.summary.clone()
        }
        AnswerStyle::Diagnostic => {
            let mut text = response.summary.clone();

            // Add details as bullets if present
            if !response.details.is_empty() {
                text.push('\n');
                for detail in &response.details {
                    text.push_str(&format!("\n- {}", detail));
                }
            }

            // Add actions if present
            if !response.actions.is_empty() {
                text.push_str("\n\nSuggested actions:");
                for action in &response.actions {
                    text.push_str(&format!("\n- {}", action.description));
                    if let Some(cmd) = &action.command {
                        text.push_str(&format!("\n  `{}`", cmd));
                    }
                }
            }

            text
        }
        AnswerStyle::Honest => {
            // Be honest about failure
            match response.status {
                StrictStatus::Failed => {
                    format!("I couldn't answer this. {}", response.summary)
                }
                StrictStatus::Partial => {
                    format!("{}\n\n(Partial answer - some data was unavailable)", response.summary)
                }
                StrictStatus::Ok => response.summary.clone(),
            }
        }
    }
}

impl StrictSpecialistResponse {
    /// Get status as string
    pub fn status_str(&self) -> &'static str {
        match self.status {
            StrictStatus::Ok => "ok",
            StrictStatus::Partial => "partial",
            StrictStatus::Failed => "failed",
        }
    }
}

/// Quick format for simple queries (one-liner + evidence)
pub fn quick_format(response: &StrictSpecialistResponse) -> String {
    let mut output = response.summary.clone();

    if !response.evidence.is_empty() {
        let probes: Vec<&str> = response.evidence.iter().map(|e| e.probe.as_str()).collect();
        output.push_str(&format!("\n\nEvidence: {}", probes.join(", ")));
    }

    output
}

/// Format error for user display
pub fn format_error(error: &str, suggestions: &[String]) -> String {
    let mut output = format!("I encountered an error: {}", error);

    if !suggestions.is_empty() {
        output.push_str("\n\nYou can try:");
        for s in suggestions.iter().take(3) {
            output.push_str(&format!("\n- {}", s));
        }
    }

    output
}

/// Format timeout for user display
pub fn format_timeout(elapsed_secs: u64) -> String {
    format!(
        "The specialist timed out after {}s. This might be a complex query or the system is busy.",
        elapsed_secs
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strict_contract::EvidenceItem;

    #[test]
    fn test_shape_concise_answer() {
        let response = StrictSpecialistResponse::ok("DSK-001", "query_metric", "Available memory: 17.0 GiB", 0.95)
            .with_evidence("memory_info", "MemAvailable: 17892232 kB");

        let shaped = shape_answer(&response, TranslatorIntent::QueryMetric, true, false);

        assert_eq!(shaped.text, "Available memory: 17.0 GiB");
        assert!(shaped.evidence_line.is_some());
        assert!(shaped.is_resolved);
    }

    #[test]
    fn test_shape_failed_answer() {
        let response = StrictSpecialistResponse::failed("DSK-001", "query_metric", "No probe data available");

        let shaped = shape_answer(&response, TranslatorIntent::QueryMetric, false, false);

        assert!(shaped.text.contains("couldn't answer"));
        assert!(!shaped.is_resolved);
    }

    #[test]
    fn test_quick_format() {
        let response = StrictSpecialistResponse::ok("DSK-001", "check_status", "You have 0 failed services.", 0.95)
            .with_evidence("failed_services", "No failed units");

        let output = quick_format(&response);
        assert!(output.contains("0 failed services"));
        assert!(output.contains("Evidence:"));
    }
}
