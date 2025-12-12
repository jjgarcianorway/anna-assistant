//! Evidence Integration - Bridge between evidence pipeline and specialist (v0.0.410).
//!
//! This module integrates the evidence engine into the specialist pipeline:
//! 1. Converts translator output to evidence request
//! 2. Runs evidence pipeline (probes, docs, knowledge)
//! 3. Formats evidence bundle for specialist consumption
//!
//! The goal: Specialist sees structured evidence, not raw chaos.

use anna_shared::evidence_engine::{EvidenceBundle, EvidenceDomain, EvidenceIntent};
use anna_shared::evidence_pipeline::{
    check_instant_answer, run_evidence_pipeline, InstantAnswer, PipelineResult,
};
use anna_shared::rpc::{ProbeResult, SpecialistDomain, TranslatorTicket};
use std::collections::HashMap;
use tracing::{debug, info};

/// Result of evidence integration
pub struct EvidenceIntegrationResult {
    /// The evidence bundle for specialist
    pub bundle: EvidenceBundle,
    /// If instant answer was found, bypass LLM
    pub instant_answer: Option<String>,
    /// Tags extracted from translator
    pub tags: Vec<String>,
    /// Evidence domain
    pub domain: EvidenceDomain,
    /// Evidence intent
    pub intent: EvidenceIntent,
}

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

/// Format evidence bundle for specialist prompt
/// This is the key integration point - creates the "evidence" section
pub fn format_evidence_for_prompt(bundle: &EvidenceBundle) -> String {
    let mut output = String::new();

    // Format probes section
    if !bundle.probes.is_empty() {
        output.push_str("## Probe Evidence\n\n");
        for probe in &bundle.probes {
            output.push_str(&format!(
                "### {}\n**Summary:** {}\n**Output:**\n```\n{}\n```\n\n",
                probe.id, probe.summary, probe.excerpt
            ));
        }
    }

    // Format docs section
    if !bundle.docs.is_empty() {
        output.push_str("## Documentation\n\n");
        for doc in &bundle.docs {
            output.push_str(&format!(
                "### {} ({})\n{}\n\n",
                doc.title, doc.source, doc.snippet
            ));
        }
    }

    // Format recipe matches
    if !bundle.recipes.is_empty() {
        output.push_str("## Relevant Recipes\n\n");
        for recipe in &bundle.recipes {
            output.push_str(&format!(
                "- **{}** ({}% confidence): {}\n",
                recipe.title, recipe.confidence, recipe.summary
            ));
        }
    }

    output
}

/// Build probe map for specialist input (backward compatible)
pub fn evidence_to_probe_map(bundle: &EvidenceBundle) -> HashMap<String, String> {
    let mut probes = HashMap::new();

    for evidence in &bundle.probes {
        // Use probe ID as key, excerpt as value
        let key = evidence.id.replace("probe:", "");
        probes.insert(key, evidence.excerpt.clone());
    }

    probes
}

/// Build enhanced input with evidence for specialist
pub fn build_enhanced_specialist_input(
    ticket_id: &str,
    domain: &str,
    intent: &str,
    question: &str,
    bundle: &EvidenceBundle,
    context_claims: Option<String>, // Added context_claims parameter
) -> String {
    let probes_json = evidence_to_probe_map(bundle);

    // Build docs array
    let docs: Vec<serde_json::Value> = bundle
        .docs
        .iter()
        .map(|d| {
            serde_json::json!({
                "source": d.source.to_string(),
                "title": d.title,
                "snippet": d.snippet
            })
        })
        .collect();

    // Build recipes array
    let recipes: Vec<serde_json::Value> = bundle
        .recipes
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "title": r.title,
                "confidence": r.confidence,
                "actions": r.actions
            })
        })
        .collect();

    let mut input = serde_json::json!({
        "ticket_id": ticket_id,
        "domain": domain,
        "intent": intent,
        "question": question,
        "probes": probes_json,
        "docs": docs,
        "recipes": recipes,
        "metadata": {
            "probes_run": bundle.metadata.probes_run,
            "docs_searched": bundle.metadata.docs_searched,
            "gather_time_ms": bundle.metadata.gather_time_ms
        }
    });

    if let Some(claims) = context_claims {
        input["context_claims"] = serde_json::Value::String(claims);
    }

    serde_json::to_string_pretty(&input).unwrap_or_else(|_| "{}".to_string())
}

/// Convert SpecialistDomain to EvidenceDomain
fn specialist_to_evidence_domain(domain: SpecialistDomain) -> EvidenceDomain {
    match domain {
        SpecialistDomain::System => EvidenceDomain::System,
        SpecialistDomain::Boot => EvidenceDomain::Boot,
        SpecialistDomain::Services => EvidenceDomain::Services,
        SpecialistDomain::Network => EvidenceDomain::Network,
        SpecialistDomain::Storage => EvidenceDomain::Storage,
        SpecialistDomain::Packages => EvidenceDomain::Packages,
        SpecialistDomain::Audio => EvidenceDomain::Audio,
        SpecialistDomain::Display => EvidenceDomain::Display,
        SpecialistDomain::Desktop => EvidenceDomain::Desktop,
        SpecialistDomain::Security => EvidenceDomain::Security,
    }
}

/// Convert query intent string to EvidenceIntent
fn query_intent_to_evidence_intent(intent: &str) -> EvidenceIntent {
    match intent.to_lowercase().as_str() {
        "question" | "querymetric" => EvidenceIntent::Diagnose,
        "investigate" | "diagnose" => EvidenceIntent::Diagnose,
        "request" | "configure" => EvidenceIntent::Configure,
        "list" | "checkstatus" => EvidenceIntent::Inspect,
        "explain" => EvidenceIntent::Explain,
        _ => EvidenceIntent::Diagnose,
    }
}

/// Build tags from translator ticket and question
fn build_tags_from_ticket(ticket: &TranslatorTicket, question: &str) -> Vec<String> {
    let mut tags = Vec::new();

    // Add probes as tags (they often indicate topic)
    for probe in &ticket.needs_probes {
        // Extract key terms from probe commands
        if let Some(tag) = extract_tag_from_probe(probe) {
            tags.push(tag);
        }
    }

    // Extract keywords from question
    let keywords = extract_keywords_from_question(question);
    tags.extend(keywords);

    // Deduplicate
    tags.sort();
    tags.dedup();

    tags
}

/// Extract tag from probe command
fn extract_tag_from_probe(probe: &str) -> Option<String> {
    // Simple extraction - get the main command/topic
    let parts: Vec<&str> = probe.split_whitespace().collect();

    match parts.first()? {
        &"free" | &"memory" => Some("memory".to_string()),
        &"df" | &"lsblk" => Some("disk".to_string()),
        &"ip" | &"ss" | &"netstat" => Some("network".to_string()),
        &"systemctl" => {
            if probe.contains("status") {
                parts.get(2).map(|s| s.to_string())
            } else {
                Some("services".to_string())
            }
        }
        &"pacman" => Some("packages".to_string()),
        &"pactl" | &"wpctl" => Some("audio".to_string()),
        &"cat" | &"ls" => {
            // Try to extract config topic from path
            if probe.contains(".config") {
                Some("config".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract keywords from question
fn extract_keywords_from_question(question: &str) -> Vec<String> {
    let stopwords = [
        "is", "my", "do", "i", "have", "what", "how", "much", "the", "a", "an",
    ];

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

/// Build minimal bundle from existing probes
fn bundle_from_existing_probes(ticket_id: &str, probes: &[ProbeResult]) -> EvidenceBundle {
    use anna_shared::evidence_engine::ProbeEvidence;

    let mut bundle = EvidenceBundle::new(ticket_id);

    for probe in probes {
        if probe.exit_code == 0 && !probe.stdout.is_empty() {
            let evidence = ProbeEvidence::new(
                &format!("probe:{}", sanitize_probe_name(&probe.command)),
                &probe.command,
                &summarize_probe_output(&probe.stdout),
                &truncate(&probe.stdout, 400),
            )
            .with_exit_code(probe.exit_code);

            bundle.add_probe(evidence);
        }
    }

    bundle
}

/// Merge evidence bundle with existing probes
fn merge_with_existing_probes(
    mut bundle: EvidenceBundle,
    probes: &[ProbeResult],
) -> EvidenceBundle {
    use anna_shared::evidence_engine::ProbeEvidence;

    // Add existing probes that aren't already in the bundle
    for probe in probes {
        let probe_id = format!("probe:{}", sanitize_probe_name(&probe.command));

        if !bundle.probes.iter().any(|p| p.id == probe_id) {
            if probe.exit_code == 0 && !probe.stdout.is_empty() {
                let evidence = ProbeEvidence::new(
                    &probe_id,
                    &probe.command,
                    &summarize_probe_output(&probe.stdout),
                    &truncate(&probe.stdout, 400),
                )
                .with_exit_code(probe.exit_code);

                bundle.add_probe(evidence);
            }
        }
    }

    bundle
}

/// Sanitize probe name for use as ID
fn sanitize_probe_name(command: &str) -> String {
    command
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .replace(['/', '-'], "_")
}

/// Generate summary from probe output
fn summarize_probe_output(output: &str) -> String {
    // Take first non-empty line as summary
    output
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| truncate(l, 80))
        .unwrap_or_else(|| "Output available".to_string())
}

/// Truncate string
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords_from_question("do I have swap enabled?");
        assert!(keywords.contains(&"swap".to_string()));
        assert!(keywords.contains(&"enabled".to_string()));
    }

    #[test]
    fn test_extract_tag_from_probe() {
        assert_eq!(
            extract_tag_from_probe("free -h"),
            Some("memory".to_string())
        );
        assert_eq!(extract_tag_from_probe("df -h"), Some("disk".to_string()));
        assert_eq!(
            extract_tag_from_probe("pacman -Q vim"),
            Some("packages".to_string())
        );
    }

    #[test]
    fn test_sanitize_probe_name() {
        assert_eq!(sanitize_probe_name("df -h /"), "df");
        assert_eq!(sanitize_probe_name("systemctl status sshd"), "systemctl");
    }
}
