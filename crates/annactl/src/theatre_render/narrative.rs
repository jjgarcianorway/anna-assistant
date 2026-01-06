//! Theatre narrative building (v0.0.831).
//!
//! v0.0.202: Initial version.
//! v0.0.318: Deduplicate escalation messages.
//! v0.0.831: Process Message events from transcript for internal comms.

use anna_shared::roster::person_by_display_name;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::theatre::{describe_check, NarrativeBuilder, NarrativeSegment, Speaker};
use anna_shared::transcript::{Actor, TranscriptEventKind};

use super::helpers::{probe_id_from_command, team_from_domain};

/// Build narrative from result
/// v0.0.107: Uses assigned_staff from result for internal comms
/// v0.0.318: Deduplicate escalation messages - only show final review
/// v0.0.831: Process Message events from transcript for fly-on-the-wall view
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

    // v0.0.831: Also collect Message events for internal comms display
    let mut internal_messages: Vec<(Actor, String)> = Vec::new();

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
            // v0.0.831: Collect Message events for fly-on-the-wall display
            TranscriptEventKind::Message { text } => {
                // Only collect internal messages (not from user, not final answer)
                if event.from != Actor::You {
                    internal_messages.push((event.from.clone(), text.clone()));
                }
            }
            _ => {}
        }
    }

    // v0.0.831: If we have internal messages from core_loop, show them
    if show_internal && !internal_messages.is_empty() {
        for (actor, text) in &internal_messages {
            // Skip very long messages (likely final answers shown separately)
            // Internal comms should be short status updates
            if text.len() > 150 {
                continue;
            }

            let segment = match actor {
                Actor::Anna => NarrativeSegment::anna_internal(text.clone()),
                Actor::Specialist => {
                    // Try to determine which specialist from assigned_staff
                    if let Some(ref staff) = result.assigned_staff {
                        // Extract name from "Name (Role)" format
                        if let Some(name_end) = staff.find(" (") {
                            let name = &staff[..name_end];
                            if let Some(profile) = person_by_display_name(name) {
                                NarrativeSegment::team_member(
                                    team,
                                    profile.tier,
                                    text.clone(),
                                )
                            } else {
                                NarrativeSegment {
                                    speaker: Speaker::TeamMember {
                                        name: name.to_string(),
                                        role: "Specialist".to_string(),
                                        team: domain_str.clone(),
                                    },
                                    text: text.clone(),
                                    delay_ms: 100,
                                    internal: true,
                                    metadata: None,
                                }
                            }
                        } else {
                            NarrativeSegment::anna_internal(text.clone())
                        }
                    } else {
                        NarrativeSegment::anna_internal(text.clone())
                    }
                }
                _ => NarrativeSegment::anna_internal(text.clone()),
            };
            builder.push_segment(segment);
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
