//! Theatre narrative building (v0.0.202).

use anna_shared::rpc::ServiceDeskResult;
use anna_shared::theatre::{describe_check, NarrativeBuilder, NarrativeSegment};
use anna_shared::transcript::TranscriptEventKind;

use super::helpers::{probe_id_from_command, team_from_domain};

/// Build narrative from result
/// v0.0.107: Uses assigned_staff from result for internal comms
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

    // Check for junior/senior review events
    let mut had_escalation = false;

    for event in &result.transcript.events {
        match &event.kind {
            TranscriptEventKind::JuniorReview {
                score, verified, ..
            } => {
                if show_internal {
                    builder.add_junior_review(team, *verified, *score);
                }
            }
            TranscriptEventKind::SeniorEscalation { successful, reason } => {
                had_escalation = true;
                if show_internal {
                    if let Some(r) = reason {
                        builder.add_escalation(team, r);
                    }
                    if *successful {
                        builder.add_senior_response(team, "I've reviewed it. Here's what I found.");
                    }
                }
            }
            TranscriptEventKind::TeamReview { reviewer, .. } => {
                if show_internal && reviewer == "senior" {
                    // Senior was involved
                    had_escalation = true;
                }
            }
            _ => {}
        }
    }

    // Add wait apology if escalation happened
    if had_escalation && show_internal {
        builder.add_wait_apology();
    }

    builder.build()
}
