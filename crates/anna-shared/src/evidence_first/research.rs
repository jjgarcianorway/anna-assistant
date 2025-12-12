//! Research Loop - Evidence-First Research (v0.0.435).
//!
//! Anna follows a deterministic loop:
//! 1. Select probes based on intent keywords
//! 2. Execute probes and collect evidence
//! 3. Retrieve relevant documentation
//! 4. Synthesize answer with citations
//!
//! Max 2 iterations before giving best-effort answer.

use super::citations::{Citation, CitationStore, EvidenceId};
use super::primitives::PrimitiveLibrary;
use super::probe_plan::{ProbeExecutor, ProbeOutput, ProbePlan};
use super::sources::{HelpTextSource, KnowledgeSource, ManPageSource};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
            max_iterations: super::MAX_RESEARCH_ITERATIONS,
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

/// The research loop executor.
pub struct ResearchLoop {
    /// Primitive library.
    library: PrimitiveLibrary,
    /// Probe executor.
    executor: ProbeExecutor,
}

impl ResearchLoop {
    /// Create a new research loop.
    pub fn new() -> Self {
        Self {
            library: PrimitiveLibrary::default(),
            executor: ProbeExecutor::new(),
        }
    }

    /// Execute research for a plan.
    pub fn execute(&self, plan: &mut ResearchPlan) -> ResearchResult {
        let mut result = ResearchResult::new(&plan.ticket_id);

        // Build probe plan from keywords if not already done
        if plan.probe_plan.is_empty() {
            plan.build_probe_plan(&self.library);
        }

        // Main research loop
        while plan.can_iterate() {
            plan.next_iteration();
            result.iterations = plan.iteration;

            // Step 1: Execute probes
            let outputs = self
                .executor
                .execute_with_citations(&plan.probe_plan, &mut result.citations);
            result.probe_outputs.extend(outputs);

            // Step 2: Retrieve documentation
            let docs = self.retrieve_docs(plan, &mut result.citations);
            result.docs_retrieved.extend(docs);

            // Step 3: Check if we have enough evidence
            if result.evidence_count() >= 2 {
                result.complete = true;
                break;
            }

            // Step 4: Expand search if needed
            self.expand_search(plan, &result);
        }

        // Mark complete if we've done all iterations
        if !result.complete && result.evidence_count() > 0 {
            result.complete = true;
        }

        result
    }

    /// Retrieve documentation for commands in plan.
    fn retrieve_docs(&self, plan: &ResearchPlan, store: &mut CitationStore) -> Vec<DocResult> {
        let mut results = Vec::new();

        for command in &plan.commands {
            // Try man page first
            if let Some(doc) = self.retrieve_man_page(command, &plan.keywords, store) {
                results.push(doc);
                continue;
            }

            // Fall back to --help
            if let Some(doc) = self.retrieve_help_text(command, &plan.keywords, store) {
                results.push(doc);
            }
        }

        results
    }

    /// Retrieve and search a man page.
    fn retrieve_man_page(
        &self,
        command: &str,
        keywords: &[String],
        store: &mut CitationStore,
    ) -> Option<DocResult> {
        let mut source = ManPageSource::new(command);

        if source.retrieve().is_err() {
            return None;
        }

        let evidence_id = EvidenceId::man(command);
        let content = source.content.as_ref()?;

        // Add raw evidence
        store.add_evidence(
            evidence_id.clone(),
            KnowledgeSource::ManPage(source.clone()),
            content,
        );

        // Search for keywords and add citations
        let mut excerpts = Vec::new();
        for keyword in keywords {
            for snippet in source.search(keyword) {
                excerpts.push(snippet.clone());
                store.add_citation(Citation::new(
                    evidence_id.clone(),
                    &format!("man {}", command),
                    &snippet,
                ));
            }
        }

        Some(DocResult {
            source_type: "man".to_string(),
            name: command.to_string(),
            success: true,
            excerpts,
        })
    }

    /// Retrieve and search help text.
    fn retrieve_help_text(
        &self,
        command: &str,
        keywords: &[String],
        store: &mut CitationStore,
    ) -> Option<DocResult> {
        let mut source = HelpTextSource::new(command);

        if source.retrieve().is_err() {
            return None;
        }

        let evidence_id = EvidenceId::help(command);
        let content = source.content.as_ref()?;

        // Add raw evidence
        store.add_evidence(
            evidence_id.clone(),
            KnowledgeSource::HelpText(source.clone()),
            content,
        );

        // Search for keywords and add citations
        let mut excerpts = Vec::new();
        for keyword in keywords {
            for snippet in source.search(keyword) {
                excerpts.push(snippet.clone());
                store.add_citation(Citation::new(
                    evidence_id.clone(),
                    &format!("{} {}", command, source.variant.flag()),
                    &snippet,
                ));
            }
        }

        Some(DocResult {
            source_type: "help".to_string(),
            name: command.to_string(),
            success: true,
            excerpts,
        })
    }

    /// Expand search based on initial results.
    fn expand_search(&self, plan: &mut ResearchPlan, result: &ResearchResult) {
        // Extract additional commands from probe output
        let mut new_commands: HashSet<String> = HashSet::new();

        for output in &result.probe_outputs {
            if output.success() {
                // Look for service names, commands, etc. in output
                for word in output.raw_output.split_whitespace() {
                    // Common patterns that might indicate a command
                    if word.ends_with(".service") {
                        let service = word.trim_end_matches(".service");
                        new_commands.insert(service.to_string());
                    }
                }
            }
        }

        // Add new commands that we haven't looked up yet
        for cmd in new_commands {
            if !plan.commands.contains(&cmd) && plan.commands.len() < 5 {
                plan.commands.push(cmd);
            }
        }
    }
}

impl Default for ResearchLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick research helper for common patterns.
pub struct QuickResearch;

impl QuickResearch {
    /// Research a boot-related issue.
    pub fn boot_issue(ticket_id: &str) -> ResearchResult {
        let mut plan = ResearchPlan::new(ticket_id)
            .with_keywords(vec![
                "boot".to_string(),
                "slow".to_string(),
                "startup".to_string(),
            ])
            .with_commands(vec!["systemd-analyze".to_string()]);

        let loop_ = ResearchLoop::new();
        loop_.execute(&mut plan)
    }

    /// Research a service issue.
    pub fn service_issue(ticket_id: &str, service: &str) -> ResearchResult {
        let mut plan = ResearchPlan::new(ticket_id)
            .with_keywords(vec![
                service.to_string(),
                "service".to_string(),
                "failed".to_string(),
            ])
            .with_commands(vec!["systemctl".to_string(), "journalctl".to_string()]);

        let loop_ = ResearchLoop::new();
        loop_.execute(&mut plan)
    }

    /// Research a memory issue.
    pub fn memory_issue(ticket_id: &str) -> ResearchResult {
        let mut plan = ResearchPlan::new(ticket_id)
            .with_keywords(vec![
                "memory".to_string(),
                "ram".to_string(),
                "swap".to_string(),
            ])
            .with_commands(vec!["free".to_string()]);

        let loop_ = ResearchLoop::new();
        loop_.execute(&mut plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_plan_creation() {
        let plan = ResearchPlan::new("ticket-123")
            .with_keywords(vec!["boot".to_string(), "slow".to_string()])
            .with_commands(vec!["systemctl".to_string()]);

        assert_eq!(plan.ticket_id, "ticket-123");
        assert_eq!(plan.keywords.len(), 2);
        assert_eq!(plan.commands.len(), 1);
    }

    #[test]
    fn test_research_plan_iterations() {
        let mut plan = ResearchPlan::new("test");
        assert!(plan.can_iterate());

        plan.next_iteration();
        assert!(plan.can_iterate());

        plan.next_iteration();
        assert!(!plan.can_iterate());
    }

    #[test]
    fn test_finding_confidence() {
        let unsupported = Finding::new("claim", vec![]);
        assert!(matches!(unsupported.confidence, Confidence::Unsupported));

        let medium = Finding::new("claim", vec!["ev1".to_string()]);
        assert!(matches!(medium.confidence, Confidence::Medium));

        let high = Finding::new("claim", vec!["ev1".to_string(), "ev2".to_string()]);
        assert!(matches!(high.confidence, Confidence::High));
    }

    #[test]
    fn test_research_result_counts() {
        let mut result = ResearchResult::new("test");

        result.probe_outputs.push(ProbeOutput {
            primitive_id: "test".to_string(),
            raw_output: "output".to_string(),
            parsed: None,
            exit_code: Some(0),
            execution_time_ms: 100,
            error: None,
        });

        result.docs_retrieved.push(DocResult {
            source_type: "man".to_string(),
            name: "test".to_string(),
            success: true,
            excerpts: vec![],
        });

        assert_eq!(result.evidence_count(), 2);
    }

    #[test]
    fn test_research_loop_creation() {
        let loop_ = ResearchLoop::new();
        // Just verify it can be created
        assert!(true);
    }
}
