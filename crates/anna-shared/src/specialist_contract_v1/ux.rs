//! UX Messages (Part F) - v0.0.440.
//!
//! Clean user-facing messages for specialist failures.
//! Never show garbled JSON or parse errors to user.
//!
//! Key messages:
//! - "Specialist response invalid (timeout). Falling back to evidence-only answer."
//! - "Analysis complete with limited confidence."

use super::fallback::{FallbackReason, FallbackResponse};
use super::retry::RetrySummary;
use super::ticket_state::TicketState;

/// User-facing message for various states.
#[derive(Debug, Clone)]
pub struct UxMessage {
    /// Short status line.
    pub status: String,
    /// Detailed explanation (optional).
    pub detail: Option<String>,
    /// Suggested next steps (optional).
    pub next_steps: Option<String>,
    /// Severity for UI styling.
    pub severity: UxSeverity,
}

/// Message severity for UI styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UxSeverity {
    /// Success (green).
    Success,
    /// Warning (yellow).
    Warning,
    /// Error (red).
    Error,
    /// Info (blue).
    Info,
}

impl UxMessage {
    /// Create a success message.
    pub fn success(status: &str) -> Self {
        Self {
            status: status.to_string(),
            detail: None,
            next_steps: None,
            severity: UxSeverity::Success,
        }
    }

    /// Create a warning message.
    pub fn warning(status: &str) -> Self {
        Self {
            status: status.to_string(),
            detail: None,
            next_steps: None,
            severity: UxSeverity::Warning,
        }
    }

    /// Create an error message.
    pub fn error(status: &str) -> Self {
        Self {
            status: status.to_string(),
            detail: None,
            next_steps: None,
            severity: UxSeverity::Error,
        }
    }

    /// Add detail.
    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    /// Add next steps.
    pub fn with_next_steps(mut self, steps: &str) -> Self {
        self.next_steps = Some(steps.to_string());
        self
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let mut output = self.status.clone();
        if let Some(ref detail) = self.detail {
            output.push_str(&format!(" {}", detail));
        }
        if let Some(ref steps) = self.next_steps {
            output.push_str(&format!(" Next: {}", steps));
        }
        output
    }
}

/// Generate UX message for fallback scenario.
pub fn fallback_message(reason: FallbackReason, response: &FallbackResponse) -> UxMessage {
    let status = match reason {
        FallbackReason::Timeout => {
            "Specialist response invalid (timeout). Falling back to evidence-only answer."
        }
        FallbackReason::InvalidResponse => {
            "Specialist response invalid. Falling back to evidence-only answer."
        }
        FallbackReason::RetriesExhausted => {
            "Specialist unavailable after retries. Using evidence-only answer."
        }
        FallbackReason::Unavailable => "Specialist unavailable. Using evidence-only answer.",
    };

    let mut msg = UxMessage::warning(status);

    // Add confidence context
    if response.confidence > 0.0 {
        msg = msg.with_detail(&format!(
            "Analysis complete with {:.0}% confidence.",
            response.confidence * 100.0
        ));
    } else if response.is_insufficient() {
        msg = msg.with_detail("Insufficient evidence to provide a complete answer.");
        if !response.next_probe.is_empty() {
            msg = msg.with_next_steps("Additional probes may help.");
        }
    }

    msg
}

/// Generate UX message for successful resolution.
pub fn success_message(confidence: f64) -> UxMessage {
    let status = if confidence >= 0.9 {
        "Analysis complete."
    } else if confidence >= 0.7 {
        "Analysis complete with high confidence."
    } else {
        "Analysis complete with moderate confidence."
    };

    UxMessage::success(status)
}

/// Generate UX message for ticket state.
pub fn state_message(state: TicketState) -> UxMessage {
    match state {
        TicketState::Open => UxMessage {
            status: "Processing your request...".to_string(),
            detail: None,
            next_steps: None,
            severity: UxSeverity::Info,
        },
        TicketState::Resolved => UxMessage::success("Request completed successfully."),
        TicketState::FailedProbe => {
            UxMessage::error("Could not gather required system information.")
                .with_next_steps("Check system permissions or try again.")
        }
        TicketState::FailedSpecialist => {
            UxMessage::warning("Analysis partially complete. Some details may be missing.")
        }
        TicketState::NeedClarification => UxMessage::warning("Need more information to proceed."),
        TicketState::Escalated => UxMessage::warning("This issue requires further investigation."),
    }
}

/// Generate UX message for retry summary.
pub fn retry_message(summary: &RetrySummary) -> Option<UxMessage> {
    if summary.successful {
        return None; // No message needed for success
    }

    if summary.exhausted {
        let status = if summary.timeouts > 0 {
            "Specialist timed out. Using fallback analysis."
        } else {
            "Specialist unavailable. Using fallback analysis."
        };
        return Some(UxMessage::warning(status));
    }

    None
}

/// Progress indicator for long-running operations.
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Current step.
    pub step: usize,
    /// Total steps.
    pub total: usize,
    /// Current step description.
    pub description: String,
}

impl ProgressIndicator {
    /// Create new progress indicator.
    pub fn new(total: usize) -> Self {
        Self {
            step: 0,
            total,
            description: "Starting...".to_string(),
        }
    }

    /// Advance to next step.
    pub fn advance(&mut self, description: &str) {
        self.step = (self.step + 1).min(self.total);
        self.description = description.to_string();
    }

    /// Get progress percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.step as f64 / self.total as f64) * 100.0
        }
    }

    /// Format for display.
    pub fn display(&self) -> String {
        format!(
            "[{}/{}] {} ({:.0}%)",
            self.step,
            self.total,
            self.description,
            self.percentage()
        )
    }
}

/// Standard progress steps for ticket processing.
pub fn standard_progress_steps() -> Vec<&'static str> {
    vec![
        "Analyzing request...",
        "Gathering system information...",
        "Consulting specialist...",
        "Preparing response...",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ux_message_display() {
        let msg = UxMessage::success("Analysis complete.")
            .with_detail("High confidence.")
            .with_next_steps("Review the results.");

        let display = msg.display();
        assert!(display.contains("Analysis complete"));
        assert!(display.contains("High confidence"));
        assert!(display.contains("Review the results"));
    }

    #[test]
    fn test_fallback_message_timeout() {
        let response = FallbackResponse::new("DSK-0101", "Boot time is 7.5s.", 0.7);
        let msg = fallback_message(FallbackReason::Timeout, &response);

        assert!(msg.status.contains("timeout"));
        assert!(msg.status.contains("Falling back"));
        assert_eq!(msg.severity, UxSeverity::Warning);
    }

    #[test]
    fn test_fallback_message_insufficient() {
        let mut response =
            FallbackResponse::insufficient_evidence("DSK-0101", vec!["systemd_analyze"]);
        response.next_probe.push("boot_probe".to_string());

        let msg = fallback_message(FallbackReason::RetriesExhausted, &response);
        assert!(msg.detail.unwrap().contains("Insufficient"));
        assert!(msg.next_steps.is_some());
    }

    #[test]
    fn test_success_message_confidence() {
        let high = success_message(0.95);
        assert!(high.status.contains("complete"));
        assert_eq!(high.severity, UxSeverity::Success);

        let moderate = success_message(0.6);
        assert!(moderate.status.contains("moderate"));
    }

    #[test]
    fn test_state_messages() {
        assert_eq!(state_message(TicketState::Open).severity, UxSeverity::Info);
        assert_eq!(
            state_message(TicketState::Resolved).severity,
            UxSeverity::Success
        );
        assert_eq!(
            state_message(TicketState::FailedProbe).severity,
            UxSeverity::Error
        );
    }

    #[test]
    fn test_progress_indicator() {
        let mut progress = ProgressIndicator::new(4);
        assert_eq!(progress.percentage(), 0.0);

        progress.advance("Gathering info...");
        assert_eq!(progress.step, 1);
        assert!((progress.percentage() - 25.0).abs() < 0.1);

        let display = progress.display();
        assert!(display.contains("[1/4]"));
        assert!(display.contains("Gathering info"));
    }
}
