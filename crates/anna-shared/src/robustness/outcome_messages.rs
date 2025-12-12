//! Outcome-to-message mapping (v0.0.433).
//!
//! Translates TicketOutcome to honest user-facing messages.

use super::contract::{SpecialistResult, TicketOutcome};
use serde::{Deserialize, Serialize};

/// User-facing message for an outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    /// Main message to show.
    pub text: String,
    /// Whether this is a success message.
    pub is_success: bool,
    /// Whether to show stats for this.
    pub counts_for_stats: bool,
    /// Optional debug info (only shown in debug mode).
    pub debug_hint: Option<String>,
}

/// Structured output message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeMessage {
    /// Header/title.
    pub header: String,
    /// Main body text.
    pub body: String,
    /// Technical context (may be hidden in normal mode).
    pub context: Option<String>,
    /// Steps to show user.
    pub steps: Vec<String>,
    /// Evidence summary.
    pub evidence: Vec<String>,
    /// Whether this outcome is considered resolved.
    pub resolved: bool,
    /// Whether this outcome is a failure.
    pub is_failure: bool,
    /// Footer status line.
    pub status_line: String,
}

/// Renderer for outcome messages.
pub struct OutcomeRenderer {
    /// Whether debug mode is enabled.
    debug_mode: bool,
    /// Max width for text.
    max_width: usize,
}

impl OutcomeRenderer {
    /// Create a new renderer.
    pub fn new(debug_mode: bool) -> Self {
        Self {
            debug_mode,
            max_width: 80,
        }
    }

    /// Render a SpecialistResult to user message.
    pub fn render(&self, result: &SpecialistResult) -> OutcomeMessage {
        match result.outcome {
            TicketOutcome::Success => self.render_success(result),
            TicketOutcome::Partial => self.render_partial(result),
            TicketOutcome::ClarificationRequired => self.render_clarification(result),
            TicketOutcome::Unsupported => self.render_unsupported(result),
            TicketOutcome::InternalError => self.render_internal_error(result),
            TicketOutcome::Timeout => self.render_timeout(result),
            TicketOutcome::ParseError => self.render_parse_error(result),
        }
    }

    /// Render success outcome.
    fn render_success(&self, result: &SpecialistResult) -> OutcomeMessage {
        let handler_info = match (&result.handler, &result.department) {
            (Some(h), Some(d)) => format!("{} ({})", h, d),
            (Some(h), None) => h.clone(),
            _ => "Anna".to_string(),
        };

        let steps: Vec<String> = result
            .steps
            .iter()
            .map(|s| {
                if s.command.is_empty() {
                    format!("• {}", s.description)
                } else if s.needs_sudo {
                    format!("• {} → sudo {}", s.description, s.command)
                } else {
                    format!("• {} → {}", s.description, s.command)
                }
            })
            .collect();

        let evidence: Vec<String> = result
            .evidence_refs
            .iter()
            .map(|e| e.probe_name.clone())
            .collect();

        OutcomeMessage {
            header: format!("[anna] via {}", handler_info),
            body: result.human_summary.clone(),
            context: result.diagnosis.clone(),
            steps,
            evidence,
            resolved: true,
            is_failure: false,
            status_line: format!("Confidence: {:.0}% | Resolved", result.confidence * 100.0),
        }
    }

    /// Render partial outcome.
    fn render_partial(&self, result: &SpecialistResult) -> OutcomeMessage {
        let mut message = self.render_success(result);
        message.header = "[anna] Partial answer".to_string();
        message.body = format!(
            "I have a partial answer:\n\n{}\n\n{}",
            result.human_summary,
            result
                .diagnosis
                .as_deref()
                .unwrap_or("Some information may be incomplete.")
        );
        message.status_line = format!(
            "Confidence: {:.0}% | Partial resolution",
            result.confidence * 100.0
        );
        message.resolved = true; // Still counts as resolved
        message
    }

    /// Render clarification required.
    fn render_clarification(&self, result: &SpecialistResult) -> OutcomeMessage {
        OutcomeMessage {
            header: "[anna] Clarification needed".to_string(),
            body: result.human_summary.clone(),
            context: None,
            steps: Vec::new(),
            evidence: Vec::new(),
            resolved: false, // Does not count as resolved yet
            is_failure: false,
            status_line: "Awaiting your response".to_string(),
        }
    }

    /// Render unsupported request.
    fn render_unsupported(&self, result: &SpecialistResult) -> OutcomeMessage {
        OutcomeMessage {
            header: "[anna] Cannot help with this".to_string(),
            body: if result.human_summary.is_empty() {
                "I do not support this kind of request yet. This goes beyond my current capabilities.".to_string()
            } else {
                result.human_summary.clone()
            },
            context: if self.debug_mode {
                result.error_info.clone()
            } else {
                None
            },
            steps: Vec::new(),
            evidence: Vec::new(),
            resolved: false,
            is_failure: true,
            status_line: "Unsupported request".to_string(),
        }
    }

    /// Render internal error.
    fn render_internal_error(&self, result: &SpecialistResult) -> OutcomeMessage {
        let debug_hint = if self.debug_mode {
            result.error_info.clone()
        } else {
            None
        };

        OutcomeMessage {
            header: "[anna] Something went wrong".to_string(),
            body: "Something went wrong while processing this request (internal error)."
                .to_string(),
            context: debug_hint,
            steps: Vec::new(),
            evidence: result
                .evidence_refs
                .iter()
                .map(|e| e.probe_name.clone())
                .collect(),
            resolved: false,
            is_failure: true,
            status_line: "Internal error - check logs for details".to_string(),
        }
    }

    /// Render timeout.
    fn render_timeout(&self, result: &SpecialistResult) -> OutcomeMessage {
        let stage_info = result
            .error_info
            .as_ref()
            .map(|e| format!(" ({})", e))
            .unwrap_or_default();

        OutcomeMessage {
            header: "[anna] Time limit exceeded".to_string(),
            body: format!(
                "I ran out of time while trying to resolve this{}. I could not find a reliable answer.",
                stage_info
            ),
            context: if self.debug_mode {
                result.error_info.clone()
            } else {
                None
            },
            steps: Vec::new(),
            evidence: result
                .evidence_refs
                .iter()
                .map(|e| e.probe_name.clone())
                .collect(),
            resolved: false,
            is_failure: true,
            status_line: "Timeout - please try again".to_string(),
        }
    }

    /// Render parse error.
    fn render_parse_error(&self, result: &SpecialistResult) -> OutcomeMessage {
        OutcomeMessage {
            header: "[anna] Response error".to_string(),
            body: "The AI that assists this specialist did not answer correctly. \
                   I could not use its response safely."
                .to_string(),
            context: if self.debug_mode {
                Some(format!(
                    "LLM Parse Error: {}",
                    result.error_info.as_deref().unwrap_or("unknown")
                ))
            } else {
                None
            },
            steps: Vec::new(),
            evidence: result
                .evidence_refs
                .iter()
                .map(|e| e.probe_name.clone())
                .collect(),
            resolved: false,
            is_failure: true,
            status_line: "Parse error - LLM response invalid".to_string(),
        }
    }

    /// Format as plain text.
    pub fn format_text(&self, msg: &OutcomeMessage) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push(msg.header.clone());
        lines.push("-".repeat(msg.header.len().min(self.max_width)));
        lines.push(String::new());

        // Body
        for line in msg.body.lines() {
            lines.push(wrap_text(line, self.max_width));
        }
        lines.push(String::new());

        // Context (if debug mode and present)
        if let Some(ctx) = &msg.context {
            if self.debug_mode {
                lines.push("[debug context]".to_string());
                lines.push(ctx.clone());
                lines.push(String::new());
            }
        }

        // Steps
        if !msg.steps.is_empty() {
            lines.push("Suggested actions:".to_string());
            for step in &msg.steps {
                lines.push(format!("  {}", step));
            }
            lines.push(String::new());
        }

        // Evidence
        if !msg.evidence.is_empty() {
            lines.push(format!("Evidence: {}", msg.evidence.join(", ")));
        }

        // Status line
        lines.push(String::new());
        lines.push(msg.status_line.clone());

        lines.join("\n")
    }

    /// Get simple status for the ticket.
    pub fn status_label(&self, outcome: TicketOutcome) -> &'static str {
        match outcome {
            TicketOutcome::Success => "Resolved",
            TicketOutcome::Partial => "Partially resolved",
            TicketOutcome::ClarificationRequired => "Awaiting clarification",
            TicketOutcome::Unsupported => "Not supported",
            TicketOutcome::InternalError => "Failed (internal error)",
            TicketOutcome::Timeout => "Failed (timeout)",
            TicketOutcome::ParseError => "Failed (parse error)",
        }
    }
}

impl Default for OutcomeRenderer {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Wrap text to width.
fn wrap_text(text: &str, width: usize) -> String {
    if text.len() <= width {
        return text.to_string();
    }

    let mut result = String::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&current_line);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_message() {
        let renderer = OutcomeRenderer::new(false);
        let result = SpecialistResult::success("Memory is healthy").with_confidence(0.95);

        let msg = renderer.render(&result);
        assert!(msg.resolved);
        assert!(!msg.is_failure);
        assert!(msg.body.contains("Memory is healthy"));
    }

    #[test]
    fn test_timeout_message() {
        let renderer = OutcomeRenderer::new(false);
        let result = SpecialistResult::timeout("senior_llm");

        let msg = renderer.render(&result);
        assert!(!msg.resolved);
        assert!(msg.is_failure);
        assert!(msg.body.contains("ran out of time"));
    }

    #[test]
    fn test_debug_mode_shows_context() {
        let renderer = OutcomeRenderer::new(true);
        let result = SpecialistResult::internal_error("Stack overflow");

        let msg = renderer.render(&result);
        assert!(msg.context.is_some());
        assert!(msg.context.unwrap().contains("Stack overflow"));
    }

    #[test]
    fn test_text_formatting() {
        let renderer = OutcomeRenderer::new(false);
        let result = SpecialistResult::success("Test answer");

        let msg = renderer.render(&result);
        let text = renderer.format_text(&msg);

        assert!(text.contains("[anna]"));
        assert!(text.contains("Test answer"));
        assert!(text.contains("Resolved"));
    }
}
