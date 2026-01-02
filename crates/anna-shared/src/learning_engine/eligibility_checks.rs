//! Learning eligibility checking logic (v0.0.427).
//!
//! Determines when to learn a new recipe from a ticket.
//! Only learns when:
//! - Specialist response is success/partial with high confidence
//! - Answer is grounded in probes/documentation
//! - Pattern is generalizable (not too user-specific)

use super::eligibility_extraction::extract_intent;
use super::eligibility_types::{EligibilityResult, SkipReason};
use crate::specialist_v3::{ResponseStatus, SpecialistResponse};
use crate::ticket_lifecycle::TicketRecord;

/// Check if a ticket is eligible for learning
pub fn check_eligibility(
    ticket: &TicketRecord,
    response: &SpecialistResponse,
    existing_intents: &[&str],
) -> EligibilityResult {
    // Rule 1: Status must be success or partial
    if !matches!(
        response.status,
        ResponseStatus::Success | ResponseStatus::Partial
    ) {
        return EligibilityResult {
            eligible: false,
            reason: Some(SkipReason::ErrorStatus),
            confidence: 1.0,
            suggested_id: None,
        };
    }

    // Rule 2: Confidence must be >= 0.8 (or 0.7 for partial with evidence)
    let min_confidence = if response.status == ResponseStatus::Partial {
        super::MIN_PARTIAL_CONFIDENCE
    } else {
        super::MIN_LEARN_CONFIDENCE
    };

    if response.confidence < min_confidence {
        return EligibilityResult {
            eligible: false,
            reason: Some(SkipReason::LowConfidence),
            confidence: response.confidence,
            suggested_id: None,
        };
    }

    // Rule 3: Must have probe evidence
    if response.probes_used.is_empty() && response.findings.is_empty() {
        return EligibilityResult {
            eligible: false,
            reason: Some(SkipReason::NoEvidence),
            confidence: 0.5,
            suggested_id: None,
        };
    }

    // Rule 4: Check for vague/speculative answers
    if is_vague_answer(&response.summary, &response.analysis) {
        return EligibilityResult {
            eligible: false,
            reason: Some(SkipReason::VagueAnswer),
            confidence: 0.6,
            suggested_id: None,
        };
    }

    // Rule 5: Check for user-specific content
    if is_too_specific(&ticket.user_question, &response.summary) {
        return EligibilityResult {
            eligible: false,
            reason: Some(SkipReason::TooSpecific),
            confidence: 0.7,
            suggested_id: None,
        };
    }

    // Rule 6: Check for duplicate intent
    let intent = extract_intent(&ticket.user_question);
    if existing_intents.contains(&intent.as_str()) {
        return EligibilityResult {
            eligible: false,
            reason: Some(SkipReason::DuplicateRecipe),
            confidence: 0.8,
            suggested_id: None,
        };
    }

    // All checks passed - eligible for learning
    let suggested_id = generate_recipe_id(&intent, &ticket.ticket_id);
    EligibilityResult {
        eligible: true,
        reason: None,
        confidence: response.confidence,
        suggested_id: Some(suggested_id),
    }
}

/// Check if answer is vague or speculative
pub(crate) fn is_vague_answer(summary: &str, analysis: &[String]) -> bool {
    let vague_patterns = [
        "might be",
        "could be",
        "possibly",
        "perhaps",
        "not sure",
        "unclear",
        "i don't know",
        "cannot determine",
        "try this",
        "you could try",
        "may help",
        "should work",
        "typically",
        "usually",
    ];

    let summary_lower = summary.to_lowercase();
    for pattern in &vague_patterns {
        if summary_lower.contains(pattern) {
            return true;
        }
    }

    // Check if analysis is mostly speculation
    let vague_analysis_count = analysis
        .iter()
        .filter(|a| {
            let lower = a.to_lowercase();
            vague_patterns.iter().any(|p| lower.contains(p))
        })
        .count();

    // More than half of analysis bullets are vague
    analysis.len() > 0 && vague_analysis_count > analysis.len() / 2
}

/// Check if content is too user-specific
pub(crate) fn is_too_specific(question: &str, summary: &str) -> bool {
    let specific_patterns = [
        // Hardcoded home paths
        r"/home/\w+/",
        // Temporary paths
        r"/tmp/\w+",
        // Specific project names that look unique
        r"my[_-]?project",
        r"my[_-]?app",
        // UUIDs
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
        // Very long specific paths
        r"/\w+(/\w+){5,}",
    ];

    let combined = format!("{} {}", question, summary).to_lowercase();

    for pattern in &specific_patterns {
        if regex::Regex::new(pattern)
            .map(|re| re.is_match(&combined))
            .unwrap_or(false)
        {
            return true;
        }
    }

    false
}

/// Generate a recipe ID from intent and ticket
fn generate_recipe_id(intent: &str, ticket_id: &str) -> String {
    // Use ticket prefix + intent
    let prefix = ticket_id
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .take(3)
        .collect::<String>()
        .to_lowercase();

    format!("{}-{}", intent, prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_v3::{ProbeStatus, ProbeUsed};

    fn make_response(status: ResponseStatus, confidence: f32) -> SpecialistResponse {
        SpecialistResponse {
            ticket_id: "TEST-001".to_string(),
            status,
            confidence,
            summary: "Service nginx is running".to_string(),
            probes_used: vec![ProbeUsed {
                id: "probe:systemctl".to_string(),
                status: ProbeStatus::Ok,
                description: "Check service".to_string(),
                raw_key: None,
            }],
            ..Default::default()
        }
    }

    fn make_ticket(question: &str) -> TicketRecord {
        TicketRecord::new("TEST-001", question)
    }

    #[test]
    fn test_eligible_success() {
        let ticket = make_ticket("Why is nginx service not starting?");
        let response = make_response(ResponseStatus::Success, 0.9);

        let result = check_eligibility(&ticket, &response, &[]);
        assert!(result.eligible);
        assert!(result.suggested_id.is_some());
    }

    #[test]
    fn test_not_eligible_error_status() {
        let ticket = make_ticket("Check something");
        let response = make_response(ResponseStatus::Error, 0.9);

        let result = check_eligibility(&ticket, &response, &[]);
        assert!(!result.eligible);
        assert_eq!(result.reason, Some(SkipReason::ErrorStatus));
    }

    #[test]
    fn test_not_eligible_low_confidence() {
        let ticket = make_ticket("Check something");
        let response = make_response(ResponseStatus::Success, 0.5);

        let result = check_eligibility(&ticket, &response, &[]);
        assert!(!result.eligible);
        assert_eq!(result.reason, Some(SkipReason::LowConfidence));
    }

    #[test]
    fn test_not_eligible_no_evidence() {
        let ticket = make_ticket("Check something");
        let mut response = make_response(ResponseStatus::Success, 0.9);
        response.probes_used.clear();
        response.findings.clear();

        let result = check_eligibility(&ticket, &response, &[]);
        assert!(!result.eligible);
        assert_eq!(result.reason, Some(SkipReason::NoEvidence));
    }

    #[test]
    fn test_vague_answer_detection() {
        assert!(is_vague_answer("This might be a problem", &[]));
        assert!(is_vague_answer("I'm not sure what's happening", &[]));
        assert!(!is_vague_answer("Memory usage is 45%", &[]));
    }

    #[test]
    fn test_specific_path_detection() {
        assert!(is_too_specific("/home/john/myproject/foo", ""));
        assert!(is_too_specific("", "/tmp/abc123/test"));
        assert!(!is_too_specific("check disk space on /", ""));
        assert!(!is_too_specific("check /etc/nginx/nginx.conf", ""));
    }
}
