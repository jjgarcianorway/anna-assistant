//! Main evidence integration logic (v0.0.410).
//!
//! This module integrates the evidence engine into the specialist pipeline:
//! 1. Converts translator output to evidence request
//! 2. Runs evidence pipeline (probes, docs, knowledge)
//! 3. Formats evidence bundle for specialist consumption

use super::converters::{query_intent_to_evidence_intent, specialist_to_evidence_domain};
use super::probes::{bundle_from_existing_probes, merge_with_existing_probes};
use super::tags::build_tags_from_ticket;
use super::types::EvidenceIntegrationResult;
use anna_shared::evidence_engine::EvidenceBundle;
use anna_shared::evidence_pipeline::{
    check_instant_answer, run_evidence_pipeline, InstantAnswer, PipelineResult,
};
use anna_shared::rpc::{ProbeResult, TranslatorTicket};
use tracing::{debug, info};

/// Build evidence for specialist from translator ticket
pub fn build_evidence_for_specialist(
    ticket: &TranslatorTicket,
    ticket_id: &str,
    question: &str,
    existing_probes: &[ProbeResult],
) -> EvidenceIntegrationResult {
    // Convert SpecialistDomain to EvidenceDomain
    let domain = specialist_to_evidence_domain(ticket.domain);
    let intent = query_intent_to_evidence_intent(&ticket.intent.to_string());

    // Build tags from translator output
    let tags = build_tags_from_ticket(ticket, question);

    info!(
        "Building evidence: domain={:?}, intent={:?}, tags={:?}",
        domain, intent, tags
    );

    // Check for instant answer from knowledge index
    let domain_str = domain.to_string().to_lowercase();
    let intent_str = intent.to_string().to_lowercase();

    let instant = check_instant_answer(&domain_str, &intent_str, &tags);

    match instant {
        InstantAnswer::FromPattern {
            answer, pattern_id, ..
        } => {
            info!("Instant answer from pattern {}", pattern_id);

            // Build minimal bundle with existing probes
            let bundle = bundle_from_existing_probes(ticket_id, existing_probes);

            return EvidenceIntegrationResult {
                bundle,
                instant_answer: Some(answer),
                tags,
                domain,
                intent,
            };
        }
        _ => {
            debug!("No instant answer, gathering evidence");
        }
    }

    // Run full evidence pipeline
    let result = run_evidence_pipeline(ticket_id, question, &domain_str, &intent_str, tags.clone());

    let bundle = match result {
        PipelineResult::Instant { answer, .. } => {
            // Instant path took effect during pipeline
            return EvidenceIntegrationResult {
                bundle: EvidenceBundle::new(ticket_id),
                instant_answer: Some(answer),
                tags,
                domain,
                intent,
            };
        }
        PipelineResult::Evidence {
            bundle,
            duration_ms,
        } => {
            info!("Evidence gathered in {}ms", duration_ms);
            bundle
        }
    };

    // Merge with existing probes (they might have different data)
    let merged_bundle = merge_with_existing_probes(bundle, existing_probes);

    EvidenceIntegrationResult {
        bundle: merged_bundle,
        instant_answer: None,
        tags,
        domain,
        intent,
    }
}
