//! Doc-First Workflow for Specialists (v0.0.414).
//!
//! Implements the doc-first reasoning pattern:
//! 1. Run probes to gather system facts
//! 2. Query knowledge sources for relevant documentation
//! 3. Let LLM interpret evidence and docs together
//! 4. Generate answer with citations
//!
//! The LLM's job is interpretation, NOT invention.

use crate::evidence_engine::{DocSnippet, EvidenceBundle, ProbeEvidence};
use crate::intent_policy::IntentCategory;
use crate::knowledge_executor::{build_query_from_context, query_knowledge};
use crate::knowledge_query::{KnowledgeHit, KnowledgeResult, KnowledgeSourceKind};
use serde::{Deserialize, Serialize};

/// Doc-first workflow context
#[derive(Debug, Clone)]
pub struct DocFirstContext {
    /// Ticket ID
    pub ticket_id: String,
    /// Domain classification
    pub domain: String,
    /// Intent classification
    pub intent: String,
    /// Intent category (for routing)
    pub intent_category: IntentCategory,
    /// Original user question
    pub question: String,
    /// Extracted tags
    pub tags: Vec<String>,
}

impl DocFirstContext {
    /// Create from ticket data
    pub fn new(
        ticket_id: &str,
        domain: &str,
        intent: &str,
        question: &str,
        tags: Vec<String>,
    ) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            domain: domain.to_lowercase(),
            intent: intent.to_lowercase(),
            intent_category: IntentCategory::from_domain_intent(domain, intent),
            question: question.to_string(),
            tags,
        }
    }

    /// Get recommended probes for this context
    pub fn recommended_probes(&self) -> Vec<&'static str> {
        self.intent_category.recommended_probes()
    }
}

/// Complete evidence for specialist reasoning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistEvidence {
    /// System facts from probes
    pub probe_evidence: Vec<ProbeEvidence>,
    /// Documentation from knowledge sources
    pub doc_evidence: Vec<KnowledgeHit>,
    /// Sources that were searched
    pub sources_searched: Vec<String>,
    /// Whether wiki was available
    pub wiki_available: bool,
    /// Total gathering time (ms)
    pub gather_time_ms: u64,
}

impl SpecialistEvidence {
    /// Check if we have any evidence
    pub fn has_evidence(&self) -> bool {
        !self.probe_evidence.is_empty() || !self.doc_evidence.is_empty()
    }

    /// Format for specialist prompt context
    pub fn format_for_prompt(&self) -> String {
        let mut output = String::new();

        // Probe evidence
        if !self.probe_evidence.is_empty() {
            output.push_str("=== SYSTEM FACTS (from probes) ===\n");
            for probe in &self.probe_evidence {
                output.push_str(&format!(
                    "[{}] {}\n{}\n\n",
                    probe.id, probe.summary, probe.excerpt
                ));
            }
        }

        // Documentation
        if !self.doc_evidence.is_empty() {
            output.push_str("=== DOCUMENTATION ===\n");
            for doc in &self.doc_evidence {
                output.push_str(&format!(
                    "[{}] {} ({})\n{}\n\n",
                    doc.doc_id, doc.title, doc.origin, doc.excerpt
                ));
            }
        }

        if output.is_empty() {
            output.push_str("No evidence gathered. Cannot answer reliably.\n");
        }

        output
    }

    /// Get all citation IDs
    pub fn all_citation_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.probe_evidence.iter().map(|p| p.id.clone()).collect();
        ids.extend(self.doc_evidence.iter().map(|d| d.doc_id.clone()));
        ids
    }

    /// Format evidence line for answer footer
    pub fn format_evidence_line(&self) -> String {
        let citations: Vec<String> = self.doc_evidence.iter().map(|d| d.origin.clone()).collect();

        let probe_cmds: Vec<String> = self
            .probe_evidence
            .iter()
            .map(|p| p.command.clone())
            .collect();

        let mut parts = Vec::new();
        if !probe_cmds.is_empty() {
            parts.push(probe_cmds.join(", "));
        }
        if !citations.is_empty() {
            parts.push(citations.join(", "));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("Evidence: {}", parts.join("; "))
        }
    }
}

/// Gather evidence using doc-first workflow
pub fn gather_evidence(
    context: &DocFirstContext,
    probe_results: Vec<ProbeEvidence>,
) -> SpecialistEvidence {
    let start = std::time::Instant::now();

    // Build knowledge query from context
    let query = build_query_from_context(
        &context.domain,
        &context.intent,
        &context.question,
        &context.tags,
    );

    // Execute knowledge query
    let knowledge_result = query_knowledge(&query);

    SpecialistEvidence {
        probe_evidence: probe_results,
        doc_evidence: knowledge_result.hits,
        sources_searched: knowledge_result
            .sources_searched
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        wiki_available: knowledge_result.wiki_available,
        gather_time_ms: start.elapsed().as_millis() as u64,
    }
}

/// Convert evidence bundle to specialist evidence
pub fn from_evidence_bundle(bundle: &EvidenceBundle) -> SpecialistEvidence {
    let doc_evidence: Vec<KnowledgeHit> = bundle
        .docs
        .iter()
        .map(|d| KnowledgeHit::from_doc_snippet(d))
        .collect();

    SpecialistEvidence {
        probe_evidence: bundle.probes.clone(),
        doc_evidence,
        sources_searched: bundle.metadata.docs_searched.clone(),
        wiki_available: false, // Not tracked in old bundle
        gather_time_ms: bundle.metadata.gather_time_ms,
    }
}

/// Specialist answer with citations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedAnswer {
    /// The answer text
    pub answer: String,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Citation IDs used
    pub citations: Vec<String>,
    /// Evidence line for footer
    pub evidence_line: String,
    /// Whether answer is fully grounded in evidence
    pub grounded: bool,
    /// Unverifiable claims (if any)
    pub unverifiable_claims: Vec<String>,
}

impl CitedAnswer {
    /// Create a grounded answer
    pub fn grounded(answer: &str, confidence: u8, citations: Vec<String>) -> Self {
        let evidence_line = if citations.is_empty() {
            String::new()
        } else {
            format!("Evidence: {}", citations.join(", "))
        };

        Self {
            answer: answer.to_string(),
            confidence,
            citations,
            evidence_line,
            grounded: true,
            unverifiable_claims: vec![],
        }
    }

    /// Create an ungrounded answer (shouldn't be used often)
    pub fn ungrounded(answer: &str, unverifiable: Vec<String>) -> Self {
        Self {
            answer: answer.to_string(),
            confidence: 30, // Low confidence for ungrounded
            citations: vec![],
            evidence_line: String::new(),
            grounded: false,
            unverifiable_claims: unverifiable,
        }
    }

    /// Format complete answer with evidence footer
    pub fn format_with_footer(&self) -> String {
        let mut output = self.answer.clone();

        if !self.evidence_line.is_empty() {
            output.push_str("\n\n");
            output.push_str(&self.evidence_line);
        }

        output
    }
}

/// Rules for honest answering
#[derive(Debug, Clone, Default)]
pub struct HonestyRules {
    /// Require at least one probe result
    pub require_probe: bool,
    /// Require at least one doc citation
    pub require_doc: bool,
    /// Maximum claims without evidence
    pub max_ungrounded_claims: usize,
}

impl HonestyRules {
    /// Default rules for specialist answers
    pub fn specialist() -> Self {
        Self {
            require_probe: true,
            require_doc: false,
            max_ungrounded_claims: 1,
        }
    }

    /// Strict rules for high-confidence answers
    pub fn strict() -> Self {
        Self {
            require_probe: true,
            require_doc: true,
            max_ungrounded_claims: 0,
        }
    }

    /// Check if evidence meets honesty requirements
    pub fn check(&self, evidence: &SpecialistEvidence) -> HonestyCheck {
        let mut issues = Vec::new();

        if self.require_probe && evidence.probe_evidence.is_empty() {
            issues.push("No probe evidence gathered".to_string());
        }

        if self.require_doc && evidence.doc_evidence.is_empty() {
            issues.push("No documentation evidence".to_string());
        }

        HonestyCheck {
            passed: issues.is_empty(),
            issues,
        }
    }
}

/// Result of honesty check
#[derive(Debug, Clone)]
pub struct HonestyCheck {
    /// Whether check passed
    pub passed: bool,
    /// Issues found
    pub issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_first_context() {
        let ctx = DocFirstContext::new(
            "TEST-001",
            "services",
            "diagnose",
            "why is my nginx failing",
            vec!["nginx".to_string()],
        );

        assert_eq!(ctx.domain, "services");
        assert_eq!(ctx.intent_category, IntentCategory::DiagnoseServiceFailure);
    }

    #[test]
    fn test_specialist_evidence_format() {
        let evidence = SpecialistEvidence {
            probe_evidence: vec![ProbeEvidence::new(
                "probe:systemctl_failed",
                "systemctl --failed",
                "0 failed services",
                "0 loaded units listed.",
            )],
            doc_evidence: vec![KnowledgeHit {
                doc_id: "man:systemctl".to_string(),
                kind: KnowledgeSourceKind::ManPage,
                title: "systemctl".to_string(),
                origin: "man systemctl".to_string(),
                excerpt: "Control the systemd system...".to_string(),
                relevance: 90,
                path: None,
            }],
            sources_searched: vec!["man".to_string()],
            wiki_available: false,
            gather_time_ms: 50,
        };

        let formatted = evidence.format_for_prompt();
        assert!(formatted.contains("SYSTEM FACTS"));
        assert!(formatted.contains("DOCUMENTATION"));
        assert!(formatted.contains("systemctl"));
    }

    #[test]
    fn test_cited_answer() {
        let answer = CitedAnswer::grounded(
            "You have 0 failed services.",
            95,
            vec!["man systemctl".to_string()],
        );

        assert!(answer.grounded);
        assert_eq!(answer.confidence, 95);
        assert!(!answer.evidence_line.is_empty());
    }

    #[test]
    fn test_honesty_rules() {
        let rules = HonestyRules::specialist();

        let good_evidence = SpecialistEvidence {
            probe_evidence: vec![ProbeEvidence::new("test", "test", "test", "test")],
            ..Default::default()
        };

        let check = rules.check(&good_evidence);
        assert!(check.passed);

        let empty_evidence = SpecialistEvidence::default();
        let check2 = rules.check(&empty_evidence);
        assert!(!check2.passed);
    }
}
