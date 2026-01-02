//! Research types and data structures.

use super::super::citations::CitationStore;
use super::super::primitives::PrimitiveLibrary;
use super::super::probe_plan::{ProbeOutput, ProbePlan};
use serde::{Deserialize, Serialize};

/// Research plan for a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    /// Ticket ID.
    pub ticket_id: String,
    /// Keywords extracted from intent.
    pub keywords: Vec<String>,
    /// Domains to research.
    pub domains: Vec<String>,
    /// Commands to look up documentation for.
    pub commands: Vec<String>,
    /// Probe plan.
    pub probe_plan: ProbePlan,
    /// Current iteration.
    pub iteration: usize,
    /// Max iterations.
    pub max_iterations: usize,
}

impl ResearchPlan {
    /// Create a new research plan.
    pub fn new(ticket_id: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            keywords: Vec::new(),
            domains: Vec::new(),
            commands: Vec::new(),
            probe_plan: ProbePlan::new(ticket_id),
            iteration: 0,
            max_iterations: super::super::MAX_RESEARCH_ITERATIONS,
        }
    }

    /// Add keywords from intent analysis.
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Add commands to look up.
    pub fn with_commands(mut self, commands: Vec<String>) -> Self {
        self.commands = commands;
        self
    }

    /// Build probe plan from keywords.
    pub fn build_probe_plan(&mut self, library: &PrimitiveLibrary) {
        let keyword_refs: Vec<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
        self.probe_plan.select_from_keywords(&keyword_refs, library);
    }

    /// Check if more iterations are allowed.
    pub fn can_iterate(&self) -> bool {
        self.iteration < self.max_iterations
    }

    /// Increment iteration.
    pub fn next_iteration(&mut self) {
        self.iteration += 1;
    }
}

/// Result of a research cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    /// Ticket ID.
    pub ticket_id: String,
    /// All probe outputs.
    pub probe_outputs: Vec<ProbeOutput>,
    /// Documentation retrieved.
    pub docs_retrieved: Vec<DocResult>,
    /// Citation store.
    pub citations: CitationStore,
    /// Summary of findings.
    pub findings: Vec<Finding>,
    /// Whether research is complete.
    pub complete: bool,
    /// Iterations performed.
    pub iterations: usize,
}

impl ResearchResult {
    /// Create empty result.
    pub fn new(ticket_id: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            probe_outputs: Vec::new(),
            docs_retrieved: Vec::new(),
            citations: CitationStore::new(),
            findings: Vec::new(),
            complete: false,
            iterations: 0,
        }
    }

    /// Get total evidence count.
    pub fn evidence_count(&self) -> usize {
        self.probe_outputs.iter().filter(|p| p.success()).count()
            + self.docs_retrieved.iter().filter(|d| d.success).count()
    }

    /// Get citation count.
    pub fn citation_count(&self) -> usize {
        self.citations.citation_count()
    }

    /// Format citations for display.
    pub fn format_citations(&self) -> String {
        self.citations.format_citations()
    }
}

/// Result of documentation retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocResult {
    /// Source type.
    pub source_type: String,
    /// Command or page name.
    pub name: String,
    /// Whether retrieval succeeded.
    pub success: bool,
    /// Relevant excerpts found.
    pub excerpts: Vec<String>,
}

/// A finding from research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// What was found.
    pub claim: String,
    /// Evidence supporting it.
    pub evidence_ids: Vec<String>,
    /// Confidence level.
    pub confidence: Confidence,
}

impl Finding {
    /// Create a new finding.
    pub fn new(claim: &str, evidence_ids: Vec<String>) -> Self {
        let confidence = if evidence_ids.is_empty() {
            Confidence::Unsupported
        } else if evidence_ids.len() >= 2 {
            Confidence::High
        } else {
            Confidence::Medium
        };

        Self {
            claim: claim.to_string(),
            evidence_ids,
            confidence,
        }
    }
}

/// Confidence level for a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// Multiple sources confirm.
    High,
    /// Single source confirms.
    Medium,
    /// No evidence.
    Unsupported,
}
