//! Research Loop - Evidence-First Research (v0.0.435).
//!
//! Anna follows a deterministic loop:
//! 1. Select probes based on intent keywords
//! 2. Execute probes and collect evidence
//! 3. Retrieve relevant documentation
//! 4. Synthesize answer with citations
//!
//! Max 2 iterations before giving best-effort answer.

use super::research_types::{DocResult, ResearchPlan, ResearchResult};
use super::super::citations::{Citation, CitationStore, EvidenceId};
use super::super::primitives::PrimitiveLibrary;
use super::super::probe_plan::ProbeExecutor;
use super::super::sources::{HelpTextSource, KnowledgeSource, ManPageSource};
use std::collections::HashSet;

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
