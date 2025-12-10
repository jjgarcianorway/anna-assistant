//! Theatre narrative building (v0.0.202).

use anna_shared::rpc::ServiceDeskResult;
use anna_shared::theatre::{describe_check, NarrativeBuilder, NarrativeSegment};
use anna_shared::transcript::TranscriptEventKind;

use super::helpers::{probe_id_from_command, team_from_domain};

/// Build narrative from result
/// v0.0.107: Uses assigned_staff from result for internal comms
/// v0.0.318: Deduplicate escalation messages - only show final review
pub fn build_narrative(result: &ServiceDeskResult, show_internal: bool) -> Vec<NarrativeSegment> {
    let mut builder = NarrativeBuilder::new();
    if show_internal {
        builder = builder.with_internal_comms();
    }

    let domain_str = result.domain.to_string().to_lowercase();

    // Get team from domain
    let team = team_from_domain(&domain_str);

    // v0.0.107: Add dispatch with case number if internal comms enabled
    if show_internal {
        if let Some(ref case_num) = result.case_number {
            builder.add_dispatch(team, case_num);
        }
    }

    // Check if we have probes
    let has_probes = !result.evidence.probes_executed.is_empty();
    let probe_ids: Vec<String> = result
        .evidence
        .probes_executed
        .iter()
        .map(|p| probe_id_from_command(&p.command))
        .collect();

    // Add checking narration if we ran probes
    if has_probes {
        let check_desc = describe_check(&probe_ids);
        builder.add_checking(&check_desc);
    }

    // v0.0.318: Find the FINAL review state to avoid duplicate messages
    // We only show one junior review (the final outcome) and one escalation if it happened
    let mut final_junior_verified = false;
    let mut final_junior_score = 0u8;
    let mut had_junior_review = false;
    let mut had_escalation = false;
    let mut escalation_successful = false;
    let mut escalation_reason: Option<String> = None;

    for event in &result.transcript.events {
        match &event.kind {
            TranscriptEventKind::JuniorReview {
                score, verified, ..
            } => {
                // Keep track of the latest junior review
                had_junior_review = true;
                final_junior_verified = *verified;
                final_junior_score = *score;
            }
            TranscriptEventKind::SeniorEscalation { successful, reason } => {
                had_escalation = true;
                escalation_successful = *successful;
                escalation_reason = reason.clone();
            }
            TranscriptEventKind::TeamReview { reviewer, .. } => {
                if reviewer == "senior" {
                    had_escalation = true;
                }
            }
            _ => {}
        }
    }

    // v0.0.318: Now add only the final narrative (not duplicates)
    if show_internal && had_junior_review {
        builder.add_junior_review(team, final_junior_verified, final_junior_score);
    }

    if show_internal && had_escalation {
        if let Some(ref r) = escalation_reason {
            builder.add_escalation(team, r);
        }
        if escalation_successful {
            builder.add_senior_response(team, "I've reviewed it. Here's what I found.");
        }
    }

    // Add wait apology if escalation happened
    if had_escalation && show_internal {
        builder.add_wait_apology();
    }

    builder.build()
}
