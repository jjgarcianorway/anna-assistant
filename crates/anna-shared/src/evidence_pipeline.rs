//! Evidence Pipeline - Full integration flow (v0.0.410).
//!
//! This module shows the complete pipeline from user question to evidence:
//! 1. Translator extracts domain/intent/tags
//! 2. Knowledge index checked for instant answers
//! 3. Evidence gatherer runs probes and fetches docs
//! 4. Evidence bundle formatted for specialist
//! 5. Successful answers stored for learning
//!
//! The goal: Anna answers like a senior Arch admin who checks docs first.

use crate::evidence_engine::{EvidenceBundle, EvidenceDomain, EvidenceIntent, EvidenceRequest};
use crate::evidence_gatherer::{build_evidence_request, gather_evidence};
use crate::knowledge_index::{KnowledgeIndex, LearnedFact, LearnedPattern};
use crate::recipe_candidate::{extract_pattern_keywords, RecipeCandidate, RecipeCandidateStore};
use tracing::{debug, info};

/// Result of checking for instant answers
#[derive(Debug, Clone)]
pub enum InstantAnswer {
    /// Found a trusted pattern that can answer directly
    FromPattern {
        pattern_id: String,
        answer: String,
        evidence_ids: Vec<String>,
    },
    /// Found relevant facts that might help
    FactsAvailable { facts: Vec<(String, String)> },
    /// No instant answer, need full evidence gathering
    NeedsEvidence,
}

/// Check if we can answer instantly from knowledge
pub fn check_instant_answer(domain: &str, intent: &str, tags: &[String]) -> InstantAnswer {
    let index = KnowledgeIndex::load();

    // Try trusted patterns first
    let trusted = index.find_trusted_patterns(tags, domain, intent);
    if let Some(pattern) = trusted.first() {
        // We have a trusted pattern - can answer directly
        info!(
            "Found trusted pattern: {} (used {} times)",
            pattern.id, pattern.usage_count
        );

        // Build answer from template (simplified - real version would fill placeholders)
        let answer = if pattern.answer_template.is_empty() {
            format!(
                "Based on previous experience: {}",
                pattern.keywords.join(", ")
            )
        } else {
            pattern.answer_template.clone()
        };

        return InstantAnswer::FromPattern {
            pattern_id: pattern.id.clone(),
            answer,
            evidence_ids: pattern.required_probes.clone(),
        };
    }

    // Check for relevant facts
    let domain_facts = index.facts_for_domain(domain);
    if !domain_facts.is_empty() {
        let facts: Vec<(String, String)> = domain_facts
            .into_iter()
            .filter(|f| tags.iter().any(|t| f.key.contains(t) || t.contains(&f.key)))
            .map(|f| (f.key.clone(), f.value.clone()))
            .collect();

        if !facts.is_empty() {
            return InstantAnswer::FactsAvailable { facts };
        }
    }

    InstantAnswer::NeedsEvidence
}

/// Full evidence pipeline: question → evidence bundle
pub fn run_evidence_pipeline(
    ticket_id: &str,
    question: &str,
    domain_str: &str,
    intent_str: &str,
    tags: Vec<String>,
) -> PipelineResult {
    let start = std::time::Instant::now();

    // 1. Check for instant answer
    let instant = check_instant_answer(domain_str, intent_str, &tags);

    match instant {
        InstantAnswer::FromPattern {
            pattern_id,
            answer,
            evidence_ids,
        } => {
            // Record pattern usage
            let mut index = KnowledgeIndex::load();
            if let Some(pattern) = index.patterns.get_mut(&pattern_id) {
                pattern.record_success();
                let _ = index.save();
            }

            return PipelineResult::Instant {
                answer,
                pattern_id,
                evidence_ids,
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }
        InstantAnswer::FactsAvailable { facts } => {
            debug!(
                "Found {} relevant facts, continuing to full evidence",
                facts.len()
            );
            // Continue to full evidence gathering, but include facts
        }
        InstantAnswer::NeedsEvidence => {
            debug!("No instant answer, gathering evidence");
        }
    }

    // 2. Build evidence request
    let request = build_evidence_request(ticket_id, domain_str, intent_str, question, tags);

    // 3. Gather evidence
    let bundle = gather_evidence(&request);

    PipelineResult::Evidence {
        bundle,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Result of the evidence pipeline
#[derive(Debug)]
pub enum PipelineResult {
    /// Instant answer from learned pattern
    Instant {
        answer: String,
        pattern_id: String,
        evidence_ids: Vec<String>,
        duration_ms: u64,
    },
    /// Full evidence bundle for specialist
    Evidence {
        bundle: EvidenceBundle,
        duration_ms: u64,
    },
}

impl PipelineResult {
    /// Get duration
    pub fn duration_ms(&self) -> u64 {
        match self {
            Self::Instant { duration_ms, .. } => *duration_ms,
            Self::Evidence { duration_ms, .. } => *duration_ms,
        }
    }

    /// Check if this was an instant answer
    pub fn is_instant(&self) -> bool {
        matches!(self, Self::Instant { .. })
    }

    /// Get evidence bundle (if not instant)
    pub fn bundle(&self) -> Option<&EvidenceBundle> {
        match self {
            Self::Evidence { bundle, .. } => Some(bundle),
            _ => None,
        }
    }
}

/// Record a successful answer for learning
pub fn record_success(
    ticket_id: &str,
    domain: &str,
    intent: &str,
    question: &str,
    answer: &str,
    confidence: u8,
    evidence_ids: &[String],
    used_probes: &[String],
) {
    if confidence < 70 || evidence_ids.is_empty() {
        debug!("Not learning: confidence {} or no evidence", confidence);
        return;
    }

    let keywords = extract_pattern_keywords(question);
    if keywords.is_empty() {
        return;
    }

    // Update knowledge index
    let mut index = KnowledgeIndex::load();

    // Create or update pattern
    let mut pattern = LearnedPattern::new(keywords.clone(), domain, intent);
    pattern.answer_template = answer.to_string();
    pattern.required_probes = used_probes.to_vec();
    pattern.record_success();

    index.learn_pattern(pattern);

    // Extract facts from evidence (simplified)
    // In real implementation, this would parse probe outputs

    if let Err(e) = index.save() {
        tracing::warn!("Failed to save knowledge index: {}", e);
    }

    // Also update recipe candidates
    let mut recipe_store = RecipeCandidateStore::load();
    let keyword_count = keywords.len();
    let candidate = RecipeCandidate::new(ticket_id, domain, intent, keywords)
        .with_probes(used_probes.to_vec())
        .with_evidence(evidence_ids.to_vec());

    recipe_store.add_or_confirm(candidate);
    let _ = recipe_store.save();

    info!(
        "Learned from successful answer: {} ({} keywords)",
        ticket_id, keyword_count
    );
}

/// Extract facts from evidence bundle
pub fn extract_facts_from_bundle(bundle: &EvidenceBundle, domain: &str) -> Vec<LearnedFact> {
    let mut facts = vec![];

    for probe in &bundle.probes {
        // Extract key facts based on probe type
        match probe.id.as_str() {
            "probe:pacman_count" => {
                if let Ok(count) = probe.excerpt.trim().parse::<u32>() {
                    facts.push(LearnedFact::new(
                        "package_count",
                        &count.to_string(),
                        "packages",
                    ));
                }
            }
            "probe:memory" => {
                if probe.summary.contains("Memory:") {
                    // Extract memory info
                    facts.push(LearnedFact::new(
                        "memory_summary",
                        &probe.summary,
                        "performance",
                    ));
                }
            }
            "probe:systemctl_failed" => {
                if probe.summary.contains("No failed") {
                    facts.push(LearnedFact::new("services_healthy", "true", "services"));
                }
            }
            _ => {}
        }
    }

    facts
}

/// Format evidence for display to user
pub fn format_evidence_for_user(bundle: &EvidenceBundle) -> String {
    let mut output = String::new();

    if bundle.probes.is_empty() && bundle.docs.is_empty() {
        return "No evidence gathered.".to_string();
    }

    output.push_str("Evidence:\n");

    for probe in bundle.probes.iter().take(3) {
        output.push_str(&format!("  - [{}] {}\n", probe.id, probe.summary));
    }

    for doc in bundle.docs.iter().take(2) {
        output.push_str(&format!("  - [{}] {}\n", doc.source, doc.title));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_instant_answer_no_knowledge() {
        let result =
            check_instant_answer("unknown_domain", "unknown_intent", &["unknown".to_string()]);
        assert!(matches!(result, InstantAnswer::NeedsEvidence));
    }

    #[test]
    fn test_pipeline_result() {
        let bundle = EvidenceBundle::new("TEST-001");
        let result = PipelineResult::Evidence {
            bundle,
            duration_ms: 100,
        };

        assert!(!result.is_instant());
        assert!(result.bundle().is_some());
        assert_eq!(result.duration_ms(), 100);
    }

    #[test]
    fn test_extract_pattern_keywords_integration() {
        let keywords = extract_pattern_keywords("how much disk space do I have?");
        assert!(keywords.contains(&"disk".to_string()));
        assert!(keywords.contains(&"space".to_string()));
    }
}
