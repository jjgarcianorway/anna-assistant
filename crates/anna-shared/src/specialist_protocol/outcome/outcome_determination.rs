//! Logic for determining ticket outcomes from responses.

use super::TicketOutcome;
use crate::specialist_protocol::{ResponseStatus, StrictResponse, ValidationResult};

/// Determine ticket outcome from response
pub fn determine_outcome(
    response: &StrictResponse,
    validation: &ValidationResult,
) -> TicketOutcome {
    // If validation failed seriously, it's an internal error
    if !validation.valid {
        let has_serious_error = validation.errors.iter().any(|e| {
            matches!(
                e,
                crate::specialist_protocol::ValidationError::InventedData(_)
                    | crate::specialist_protocol::ValidationError::ForbiddenPattern(_)
            )
        });

        if has_serious_error {
            return TicketOutcome::InternalError;
        }
    }

    match response.status {
        ResponseStatus::Success => {
            // Success requires evidence and valid response
            if response.evidence.probes_used.is_empty()
                && response.evidence.arch_wiki_pages.is_empty()
                && response.evidence.man_pages.is_empty()
            {
                // No evidence - downgrade to partial
                if response.confidence >= 0.8 {
                    return TicketOutcome::UsefulPartial;
                }
                return TicketOutcome::Failed;
            }

            if validation.valid && response.confidence >= 0.7 {
                TicketOutcome::Success
            } else {
                TicketOutcome::UsefulPartial
            }
        }

        ResponseStatus::Partial => {
            // Check if partial has useful content
            if is_useful_partial(response) {
                TicketOutcome::UsefulPartial
            } else {
                TicketOutcome::Failed
            }
        }

        ResponseStatus::Failure => {
            // Check if it's an honest "I don't know" vs complete failure
            if is_honest_unknown(response) {
                TicketOutcome::HonestUnknown
            } else {
                TicketOutcome::Failed
            }
        }
    }
}

/// Check if a partial response is useful
fn is_useful_partial(response: &StrictResponse) -> bool {
    // Must have some meaningful content
    if response.summary.trim().is_empty() {
        return false;
    }

    // Must have at least one fact or evidence
    let has_facts = !response.details.key_facts.is_empty();
    let has_evidence = !response.evidence.probes_used.is_empty();
    let has_diagnosis = response
        .details
        .diagnosis
        .as_ref()
        .map(|d| !d.is_empty())
        .unwrap_or(false);

    // Check for common "useless partial" patterns
    let summary_lower = response.summary.to_lowercase();
    let useless_patterns = [
        "i could not",
        "i was unable",
        "no data available",
        "cannot determine",
        "unable to determine",
    ];

    let only_says_failure =
        useless_patterns.iter().all(|p| summary_lower.contains(p)) && !has_facts && !has_evidence;

    if only_says_failure {
        return false;
    }

    // Must have meaningful confidence
    if response.confidence < 0.3 {
        return false;
    }

    has_facts || has_evidence || has_diagnosis
}

/// Check if failure is an honest "I don't know"
fn is_honest_unknown(response: &StrictResponse) -> bool {
    let summary_lower = response.summary.to_lowercase();

    // Honest unknown patterns
    let honest_patterns = [
        "i don't have",
        "i cannot determine",
        "no specialist available",
        "outside my expertise",
        "i don't know",
        "i cannot answer",
        "i lack the data",
    ];

    honest_patterns.iter().any(|p| summary_lower.contains(p))
}
