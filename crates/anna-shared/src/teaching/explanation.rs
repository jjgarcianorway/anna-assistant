//! Service Desk Explanation Engine
//!
//! Generates teaching explanations that mirror how a real service desk reasons.
//!
//! Output rules:
//! - Explain what signals would be checked
//! - Explain why those signals matter
//! - State conclusions supported by evidence
//! - Explicitly state what is unknown
//! - NEVER provide commands
//! - NEVER suggest actions
//! - NEVER say "you should"

use super::grounding::{gather_grounding, has_sufficient_grounding, report_missing_grounding};
use super::mode::{
    ConclusionConfidence, EvidenceSource, EvidencedConclusion, GroundingContext,
    StateEvidence, TeachingExplanation, TeachingOutput, TeachingQuestion,
};
use crate::monitor::IssueType;
use chrono::Utc;

/// Generate a teaching explanation for a given question.
///
/// This is the main entry point for Teaching Mode.
pub fn generate_teaching_explanation(
    question: &str,
    question_type: TeachingQuestion,
    subject: Option<&str>,
) -> TeachingOutput {
    // Only generate for teaching-routed questions
    if !question_type.routes_to_teaching() {
        return TeachingOutput {
            explanation: TeachingExplanation {
                signals_to_check: vec![],
                why_signals_matter: "This question does not require teaching output.".to_string(),
                conclusions: vec![],
                unknowns: vec!["Question routed elsewhere".to_string()],
            },
            grounding: GroundingContext::default(),
            fully_grounded: false,
        };
    }

    // Gather grounding context
    let grounding = gather_grounding(subject);
    let fully_grounded = has_sufficient_grounding(&grounding);

    // Generate explanation based on question type and grounding
    let explanation = match question_type {
        TeachingQuestion::How => generate_how_explanation(question, &grounding),
        TeachingQuestion::Why => generate_why_explanation(question, &grounding),
        TeachingQuestion::GeneralLinux => generate_general_explanation(question, &grounding),
        _ => TeachingExplanation {
            signals_to_check: vec![],
            why_signals_matter: String::new(),
            conclusions: vec![],
            unknowns: vec!["Unexpected question type for teaching".to_string()],
        },
    };

    TeachingOutput {
        explanation,
        grounding,
        fully_grounded,
    }
}

/// Generate explanation for "How" questions.
fn generate_how_explanation(question: &str, grounding: &GroundingContext) -> TeachingExplanation {
    let q = question.to_lowercase();

    // Determine what signals to check based on question content
    let mut signals = Vec::new();
    let mut why_matters = String::new();
    let mut conclusions = Vec::new();
    let mut unknowns = Vec::new();

    // Pattern: "how would you diagnose X"
    if q.contains("diagnose") || q.contains("troubleshoot") || q.contains("investigate") {
        signals.push("Recent changes in system state (baseline diffs)".to_string());
        signals.push("Related warnings or issues in the issue store".to_string());
        signals.push("Outcome of previous actions from the ledger".to_string());
        signals.push("Service states and config file integrity".to_string());

        why_matters = "These signals establish the timeline and scope of the issue. \
            A service desk correlates changes with symptoms to narrow down root cause."
            .to_string();

        // Add conclusions from grounding
        for evidence in &grounding.system_state {
            conclusions.push(EvidencedConclusion {
                conclusion: format!("Observed: {}", evidence.observation),
                evidence: evidence.source.display(),
                confidence: ConclusionConfidence::Supported,
            });
        }

        if grounding.system_state.is_empty() {
            unknowns.push("No system state evidence available to ground diagnosis".to_string());
        }
    }
    // Pattern: "how does X work"
    else if q.contains("work") || q.contains("function") {
        signals.push("Current state of the component on your system".to_string());
        signals.push("Configuration files that affect this component".to_string());
        signals.push("Related services and dependencies".to_string());

        why_matters = "Understanding how something works requires seeing it in context. \
            A service desk examines the actual state, not just documentation."
            .to_string();

        // Ground in actual system state if available
        if !grounding.system_state.is_empty() {
            conclusions.push(EvidencedConclusion {
                conclusion: "System state evidence is available for context".to_string(),
                evidence: format!("{} pieces of evidence gathered", grounding.system_state.len()),
                confidence: ConclusionConfidence::Supported,
            });
        } else {
            unknowns.push("Cannot ground explanation in current system state".to_string());
        }
    }
    // Default how question
    else {
        signals.push("Current system state relevant to the question".to_string());
        signals.push("Historical changes and outcomes".to_string());

        why_matters = "A service desk examines evidence before forming conclusions.".to_string();

        if grounding.system_state.is_empty() {
            unknowns.push("Insufficient grounding to provide specific guidance".to_string());
        }
    }

    // Add grounding gaps as unknowns
    unknowns.extend(report_missing_grounding(grounding));

    TeachingExplanation {
        signals_to_check: signals,
        why_signals_matter: why_matters,
        conclusions,
        unknowns,
    }
}

/// Generate explanation for "Why" questions.
fn generate_why_explanation(question: &str, grounding: &GroundingContext) -> TeachingExplanation {
    let q = question.to_lowercase();

    let mut signals = Vec::new();
    let mut why_matters = String::new();
    let mut conclusions = Vec::new();
    let mut unknowns = Vec::new();

    // Pattern: "why is X happening"
    if q.contains("happening") || q.contains("occurring") || q.contains("doing") {
        signals.push("Timeline of events leading to current state".to_string());
        signals.push("Correlation between changes and symptoms".to_string());
        signals.push("Service logs and error messages (if available)".to_string());

        why_matters = "Causation requires correlation in time. A service desk establishes \
            what changed before the symptom appeared to identify likely causes."
            .to_string();

        // Look for evidence of changes
        if !grounding.diffs.is_empty() {
            for diff in &grounding.diffs {
                conclusions.push(EvidencedConclusion {
                    conclusion: format!("Change detected: {}", diff),
                    evidence: "[baseline comparison]".to_string(),
                    confidence: ConclusionConfidence::Supported,
                });
            }
        } else {
            unknowns.push("No changes detected in baseline comparison".to_string());
        }
    }
    // Pattern: "why does X matter"
    else if q.contains("matter") || q.contains("important") || q.contains("care") {
        signals.push("Impact on system stability or security".to_string());
        signals.push("Dependencies that could be affected".to_string());
        signals.push("Historical issues caused by similar situations".to_string());

        why_matters = "Importance is measured by impact. A service desk evaluates \
            what could go wrong if the issue is ignored."
            .to_string();

        // Look for active issues as evidence of importance
        let issue_evidence: Vec<_> = grounding
            .system_state
            .iter()
            .filter(|e| matches!(e.source, EvidenceSource::IssueStore))
            .collect();

        if !issue_evidence.is_empty() {
            conclusions.push(EvidencedConclusion {
                conclusion: "Related issues exist in issue store".to_string(),
                evidence: format!("{} related entries", issue_evidence.len()),
                confidence: ConclusionConfidence::Partial,
            });
        }
    }
    // Default why question
    else {
        signals.push("Evidence supporting causal relationship".to_string());
        signals.push("Counterevidence that might disprove assumptions".to_string());

        why_matters = "A service desk avoids speculation. Claims require evidence.".to_string();
    }

    // Add grounding gaps
    if grounding.system_state.is_empty() {
        unknowns.push("Cannot establish causation without system state evidence".to_string());
    }
    unknowns.extend(report_missing_grounding(grounding));

    TeachingExplanation {
        signals_to_check: signals,
        why_signals_matter: why_matters,
        conclusions,
        unknowns,
    }
}

/// Generate explanation for general Linux questions.
/// Only provides teaching if tied to current system state.
fn generate_general_explanation(question: &str, grounding: &GroundingContext) -> TeachingExplanation {
    // General questions require grounding to avoid becoming a tutorial
    if grounding.system_state.is_empty() {
        return TeachingExplanation {
            signals_to_check: vec![],
            why_signals_matter: String::new(),
            conclusions: vec![],
            unknowns: vec![
                "This is a general Linux question not tied to current system state.".to_string(),
                "Teaching Mode only explains concepts when grounded in observed evidence.".to_string(),
                "For general documentation, consult the Arch Wiki or man pages.".to_string(),
            ],
        };
    }

    // We have grounding - provide explanation tied to system state
    let mut signals = Vec::new();
    let mut conclusions = Vec::new();

    signals.push("How this concept applies to your current system state".to_string());
    signals.push("Relevant configuration or state on your machine".to_string());

    let why_matters = "A service desk grounds explanations in reality. \
        Abstract concepts become meaningful when tied to what you can observe."
        .to_string();

    // Add observed evidence as conclusions
    for evidence in &grounding.system_state {
        conclusions.push(EvidencedConclusion {
            conclusion: evidence.observation.clone(),
            evidence: evidence.source.display(),
            confidence: ConclusionConfidence::Supported,
        });
    }

    TeachingExplanation {
        signals_to_check: signals,
        why_signals_matter: why_matters,
        conclusions,
        unknowns: report_missing_grounding(grounding),
    }
}

/// Generate a specific explanation for the "config changed: group" warning.
/// This is a reference implementation showing how Teaching Mode handles a real warning.
pub fn explain_group_warning() -> TeachingOutput {
    let grounding = gather_grounding(Some("group"));

    let signals = vec![
        "/etc/group file hash comparison against baseline".to_string(),
        "Package manager logs for recent operations".to_string(),
        "User/group management operations in auth.log".to_string(),
        "Timestamps: when was the file modified vs when was the warning raised".to_string(),
    ];

    let why_matters = "The /etc/group file defines group memberships for system security. \
        Changes to this file can affect:\n\
        - User access to devices (audio, video, storage)\n\
        - Service permissions (docker, libvirt)\n\
        - Administrative capabilities (wheel group)\n\n\
        A service desk examines whether the change was intentional (package update, \
        user action) or unexpected (potential issue)."
        .to_string();

    let mut conclusions = Vec::new();

    // Add conclusions from actual grounding
    for evidence in &grounding.system_state {
        conclusions.push(EvidencedConclusion {
            conclusion: evidence.observation.clone(),
            evidence: evidence.source.display(),
            confidence: ConclusionConfidence::Supported,
        });
    }

    // If we have diffs, mention them
    if !grounding.diffs.is_empty() {
        conclusions.push(EvidencedConclusion {
            conclusion: "File content differs from recorded baseline".to_string(),
            evidence: "[baseline hash mismatch]".to_string(),
            confidence: ConclusionConfidence::Supported,
        });
    }

    let mut unknowns = vec![
        "What specific change was made (without file diff)".to_string(),
        "Which process made the change (requires audit log)".to_string(),
    ];
    unknowns.extend(report_missing_grounding(&grounding));

    let explanation = TeachingExplanation {
        signals_to_check: signals,
        why_signals_matter: why_matters,
        conclusions,
        unknowns,
    };

    TeachingOutput {
        explanation,
        grounding,
        fully_grounded: has_sufficient_grounding(&gather_grounding(Some("group"))),
    }
}

//------------------------------------------------------------------------------
// EXAMPLES: Good vs Forbidden Teaching Output
//------------------------------------------------------------------------------
//
// === GOOD OUTPUT (allowed) ===
//
// User: "why is the group warning happening?"
//
// SERVICE DESK PERSPECTIVE
// ========================
//
// Signals a service desk would examine:
//   - /etc/group file hash comparison against baseline
//   - Package manager logs for recent operations
//   - User/group management operations in auth.log
//   - Timestamps: when was the file modified
//
// Why these signals matter:
//   The /etc/group file defines group memberships for system security.
//   Changes can affect user access to devices and service permissions.
//   A service desk determines if the change was intentional or unexpected.
//
// Conclusions from available evidence:
//   - Config file /etc/group differs from baseline (supported)
//     Evidence: [baseline snapshot]
//   - Active issue: Config changed: group (Warning) (supported)
//     Evidence: [issue store]
//
// What is unknown:
//   - What specific change was made (without file diff)
//   - Which process made the change (requires audit log)
//
// Evidence sources used:
//   - [baseline snapshot] /etc/group hash mismatch
//   - [issue store] Warning active since 2026-01-16 14:00
//
// [End of teaching output]
//
// === FORBIDDEN OUTPUT (never allowed) ===
//
// "You should run `cat /etc/group` to see the contents."
// "Try running `sudo usermod -aG wheel youruser` to fix it."
// "I recommend checking the file with `diff /etc/group /etc/group.pacnew`."
// "The fix is to restore from backup."
// "You need to add yourself to the group."
// "Run these commands: ..."
//
//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_non_teaching_question() {
        let output = generate_teaching_explanation(
            "what is my disk usage?",
            TeachingQuestion::Status,
            None,
        );

        assert!(!output.fully_grounded);
        assert!(output.explanation.unknowns.contains(&"Question routed elsewhere".to_string()));
    }

    #[test]
    fn test_generate_how_question() {
        let output = generate_teaching_explanation(
            "how would you diagnose this issue?",
            TeachingQuestion::How,
            None,
        );

        assert!(!output.explanation.signals_to_check.is_empty());
        assert!(!output.explanation.why_signals_matter.is_empty());
    }

    #[test]
    fn test_generate_why_question() {
        let output = generate_teaching_explanation(
            "why is this happening?",
            TeachingQuestion::Why,
            None,
        );

        assert!(!output.explanation.signals_to_check.is_empty());
        assert!(output.explanation.why_signals_matter.contains("correlation"));
    }

    #[test]
    fn test_general_question_produces_safe_output() {
        let output = generate_teaching_explanation(
            "what is systemd?",
            TeachingQuestion::GeneralLinux,
            None,
        );

        // General Linux question should produce safe output:
        // - Either redirect to docs (if no grounding)
        // - Or explain with grounding (if grounding available)
        // In both cases, no commands should appear
        let formatted = super::super::mode::format_teaching_output(&output);
        assert!(!formatted.contains("sudo "));
        assert!(!formatted.contains("You should"));
        assert!(!formatted.contains("Try running"));

        // Should end properly
        assert!(formatted.contains("[End of teaching output]"));
    }

    #[test]
    fn test_explain_group_warning() {
        let output = explain_group_warning();

        // Should have proper structure
        assert!(!output.explanation.signals_to_check.is_empty());
        assert!(output.explanation.why_signals_matter.contains("/etc/group"));
        assert!(!output.explanation.unknowns.is_empty());

        // Should NOT contain forbidden action patterns
        let formatted = super::super::mode::format_teaching_output(&output);
        assert!(!formatted.contains("You should"));
        assert!(!formatted.contains("Try running"));
        assert!(!formatted.contains("sudo "));  // Space after to avoid "sudo" in explanatory text
        assert!(!formatted.contains("I recommend"));
    }

    #[test]
    fn test_no_commands_in_output() {
        let output = generate_teaching_explanation(
            "how would you fix this?",
            TeachingQuestion::How,
            None,
        );

        let formatted = super::super::mode::format_teaching_output(&output);

        // Check for command patterns (with space to avoid false positives)
        assert!(!formatted.contains("sudo "));
        assert!(!formatted.contains("pacman "));
        assert!(!formatted.contains("systemctl "));
        assert!(!formatted.contains("cat "));
        // Don't check for " -" as it's too broad (matches list items like " - Foo")
    }
}
