//! Acknowledgment - Strictly gated resolution acknowledgment.
//!
//! ONLY output when user explicitly asks:
//! - "what changed"
//! - "why is this resolved"
//! - "what happened to the warning"
//! - Similar explicit inquiries
//!
//! Output rules:
//! - State the observed change
//! - State the attribution if known
//! - State uncertainty if attribution is unknown
//! - NO advice
//! - NO next steps
//! - NO suggestions
//! - NO "you could"

use super::attribution::{Actor, Attribution, Confidence};
use super::recognition::{Resolution, ResolutionEvent};
use regex::Regex;
use std::sync::LazyLock;

/// Patterns that indicate user is asking about a resolution.
static RESOLUTION_INQUIRY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // "what changed" patterns
        Regex::new(r"(?i)\bwhat (changed|happened)\b").unwrap(),
        Regex::new(r"(?i)\bwhy (is|was) .* (resolved|gone|fixed|cleared)\b").unwrap(),
        Regex::new(r"(?i)\bwhy did .* (go away|disappear|clear|resolve)\b").unwrap(),
        // "what happened to" patterns
        Regex::new(r"(?i)\bwhat happened to (the|my|this) (warning|issue|alert|error)\b").unwrap(),
        // "is it resolved" patterns
        Regex::new(r"(?i)\bis (the|this|that) .* (resolved|fixed|gone)\b").unwrap(),
        // "how was X resolved" patterns
        Regex::new(r"(?i)\bhow (was|did) .* (get )?(resolved|fixed)\b").unwrap(),
    ]
});

/// Check if a question is asking about a resolution.
/// This gates acknowledgment output - we only respond about resolutions if asked.
pub fn is_resolution_inquiry(question: &str) -> bool {
    let q = question.trim().to_lowercase();

    for pattern in RESOLUTION_INQUIRY_PATTERNS.iter() {
        if pattern.is_match(&q) {
            return true;
        }
    }

    false
}

/// Format a resolution acknowledgment.
///
/// Output format (strictly constrained):
/// ```text
/// RESOLUTION OBSERVED
/// -------------------
/// Issue: [original summary]
/// Status: [resolution type]
/// Attribution: [actor] ([confidence])
/// Evidence: [evidence or "none"]
///
/// [End of observation]
/// ```
///
/// FORBIDDEN outputs:
/// - Suggestions
/// - Advice
/// - "You could..."
/// - "Next steps..."
/// - Speculation about causes
/// - Praise or rewards
pub fn format_resolution_acknowledgment(
    resolution: &ResolutionEvent,
    attribution: &Attribution,
) -> String {
    let mut output = String::new();

    output.push_str("RESOLUTION OBSERVED\n");
    output.push_str("-------------------\n\n");

    // Issue
    output.push_str(&format!("Issue: {}\n", resolution.original_summary));

    // Status
    let status = match resolution.resolution {
        Resolution::IssueCleared => "Issue no longer active",
        Resolution::ReturnedToBaseline => "State returned to baseline",
        Resolution::NewBaselineEstablished => "New baseline established",
    };
    output.push_str(&format!("Status: {}\n", status));

    // Attribution
    let actor_str = match attribution.actor {
        Actor::Anna => "Anna action",
        Actor::User => "External action (not Anna)",
        Actor::Unknown => "Unknown",
    };
    let confidence_str = match attribution.confidence {
        Confidence::High => "high confidence",
        Confidence::Medium => "medium confidence",
        Confidence::Low => "low confidence",
        Confidence::None => "insufficient evidence",
    };
    output.push_str(&format!("Attribution: {} ({})\n", actor_str, confidence_str));

    // Evidence
    output.push_str(&format!(
        "Evidence: {}\n",
        resolution.evidence
    ));

    // Timestamps (if available)
    if let Some(detected_at) = resolution.issue_detected_at {
        output.push_str(&format!(
            "\nIssue first detected: {}\n",
            detected_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
    }
    output.push_str(&format!(
        "Resolution detected: {}\n",
        resolution.detected_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Terminator - explicitly signals end, no continuation
    output.push_str("\n[End of observation]\n");

    output
}

/// Format a "no resolution found" response.
/// Used when user asks about a resolution but none is detected.
pub fn format_no_resolution(subject: &str) -> String {
    format!(
        "RESOLUTION STATUS\n\
         -----------------\n\n\
         Subject: {}\n\
         Status: No resolution detected\n\
         Note: Issue may still be active or not tracked\n\n\
         [End of observation]\n",
        subject
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::monitor::IssueType;

    #[test]
    fn test_resolution_inquiry_detection() {
        // Should match
        assert!(is_resolution_inquiry("what changed?"));
        assert!(is_resolution_inquiry("why is the warning resolved?"));
        assert!(is_resolution_inquiry("what happened to the group warning?"));
        assert!(is_resolution_inquiry("why did the error go away?"));
        assert!(is_resolution_inquiry("is the ssh issue fixed?"));

        // Should NOT match
        assert!(!is_resolution_inquiry("what is my disk usage?"));
        assert!(!is_resolution_inquiry("install neovim"));
        assert!(!is_resolution_inquiry("how do I configure ssh?"));
    }

    #[test]
    fn test_acknowledgment_format() {
        let resolution = ResolutionEvent {
            id: "test-123".to_string(),
            issue_type: IssueType::ConfigChanged,
            original_summary: "Config changed: group".to_string(),
            resolution: Resolution::IssueCleared,
            detected_at: Utc::now(),
            issue_detected_at: Some(Utc::now()),
            evidence: "File hash matches baseline".to_string(),
        };

        let attribution = Attribution {
            actor: Actor::User,
            confidence: Confidence::Medium,
            evidence: Some("User asked then resolved".to_string()),
            reason: "External resolution".to_string(),
        };

        let output = format_resolution_acknowledgment(&resolution, &attribution);

        // Check structure
        assert!(output.contains("RESOLUTION OBSERVED"));
        assert!(output.contains("Config changed: group"));
        assert!(output.contains("External action (not Anna)"));
        assert!(output.contains("medium confidence"));
        assert!(output.contains("[End of observation]"));

        // Check forbidden content is NOT present
        assert!(!output.contains("You could"));
        assert!(!output.contains("you should"));
        assert!(!output.contains("next step"));
        assert!(!output.contains("recommend"));
    }

    #[test]
    fn test_no_resolution_format() {
        let output = format_no_resolution("group warning");

        assert!(output.contains("RESOLUTION STATUS"));
        assert!(output.contains("group warning"));
        assert!(output.contains("No resolution detected"));
        assert!(output.contains("[End of observation]"));
    }
}
