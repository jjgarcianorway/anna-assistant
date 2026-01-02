//! Research Plan and Citations - Core Types - v0.0.443.
//!
//! Core types for research planning and citations:
//! - ResearchPlan: Plan generated before specialist
//! - ResearchConstraints: Constraints for research
//! - Citation: Citation reference
//! - CitedAnswer: Answer with citations
//! - ResearchResult: Result containing fetched sources

use serde::{Deserialize, Serialize};

use super::providers::{commands_for_intent, SourceContent, SourceRequest, SourceType};

/// Research plan generated before calling specialist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    /// Goal of the research.
    pub goal: String,
    /// Required facts.
    pub required_facts: Vec<String>,
    /// Probes to execute.
    pub probes: Vec<String>,
    /// Sources to fetch.
    pub sources: Vec<SourceRequest>,
    /// Constraints.
    #[serde(default)]
    pub constraints: ResearchConstraints,
}

/// Research constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchConstraints {
    /// Only use offline sources.
    pub offline_only: bool,
    /// Maximum sources to fetch.
    pub max_sources: Option<usize>,
    /// Timeout per source in ms.
    pub source_timeout_ms: Option<u64>,
}

impl ResearchPlan {
    /// Create new plan.
    pub fn new(goal: &str) -> Self {
        Self {
            goal: goal.to_string(),
            required_facts: Vec::new(),
            probes: Vec::new(),
            sources: Vec::new(),
            constraints: ResearchConstraints::default(),
        }
    }

    /// Add required fact.
    pub fn require_fact(mut self, fact: &str) -> Self {
        self.required_facts.push(fact.to_string());
        self
    }

    /// Add probe.
    pub fn add_probe(mut self, probe: &str) -> Self {
        self.probes.push(probe.to_string());
        self
    }

    /// Add source.
    pub fn add_source(mut self, source: SourceRequest) -> Self {
        self.sources.push(source);
        self
    }

    /// Set offline only.
    pub fn offline_only(mut self) -> Self {
        self.constraints.offline_only = true;
        self
    }

    /// Generate plan from intent.
    pub fn from_intent(intent: &str, goal: &str) -> Self {
        let mut plan = Self::new(goal);

        // Get canonical commands for this intent
        if let Some(cmds) = commands_for_intent(intent) {
            // Add man pages for each command
            for cmd in &cmds.commands {
                plan.sources
                    .push(SourceRequest::man(&format!("{}(1)", cmd), "DESCRIPTION"));
                plan.sources.push(SourceRequest::help(cmd, ""));
            }

            // Add wiki pages (optional)
            for page in &cmds.wiki_pages {
                plan.sources
                    .push(SourceRequest::arch_wiki(page, "").optional());
            }
        }

        plan
    }

    /// Get required sources.
    pub fn required_sources(&self) -> Vec<&SourceRequest> {
        self.sources.iter().filter(|s| s.required).collect()
    }

    /// Get optional sources.
    pub fn optional_sources(&self) -> Vec<&SourceRequest> {
        self.sources.iter().filter(|s| !s.required).collect()
    }
}

/// Citation reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Citation ID (e.g., "S1", "E1").
    pub id: String,
    /// Source type.
    pub source_type: SourceType,
    /// Source identifier.
    pub source_id: String,
    /// Section/query.
    pub section: String,
    /// Excerpt (max 200 chars).
    pub excerpt: Option<String>,
}

impl Citation {
    /// Create documentation citation.
    pub fn source(id: &str, source_type: SourceType, source_id: &str, section: &str) -> Self {
        Self {
            id: id.to_string(),
            source_type,
            source_id: source_id.to_string(),
            section: section.to_string(),
            excerpt: None,
        }
    }

    /// Create evidence citation.
    pub fn evidence(id: &str, probe: &str, output_excerpt: &str) -> Self {
        Self {
            id: id.to_string(),
            source_type: SourceType::Probe,
            source_id: probe.to_string(),
            section: String::new(),
            excerpt: Some(super::research::truncate(output_excerpt, 200)),
        }
    }

    /// Add excerpt.
    pub fn with_excerpt(mut self, excerpt: &str) -> Self {
        self.excerpt = Some(super::research::truncate(excerpt, 200));
        self
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let source_label = match self.source_type {
            SourceType::Man => format!("man {}", self.source_id),
            SourceType::Help => format!("{} output", self.source_id),
            SourceType::ArchWiki => format!("Arch Wiki \"{}\"", self.source_id),
            SourceType::LocalConfig => format!("config {}", self.source_id),
            SourceType::Probe => format!("probe: {}", self.source_id),
        };

        let section = if self.section.is_empty() {
            String::new()
        } else {
            format!(" section \"{}\"", self.section)
        };

        format!("[{}] {}{}", self.id, source_label, section)
    }
}

/// Answer with citations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedAnswer {
    /// The answer text.
    pub answer: String,
    /// Documentation sources used.
    pub sources: Vec<Citation>,
    /// Evidence from this machine.
    pub evidence: Vec<Citation>,
    /// Confidence.
    pub confidence: f64,
}

impl CitedAnswer {
    /// Create new cited answer.
    pub fn new(answer: &str, confidence: f64) -> Self {
        Self {
            answer: answer.to_string(),
            sources: Vec::new(),
            evidence: Vec::new(),
            confidence,
        }
    }

    /// Add source citation.
    pub fn cite_source(mut self, citation: Citation) -> Self {
        self.sources.push(citation);
        self
    }

    /// Add evidence citation.
    pub fn cite_evidence(mut self, citation: Citation) -> Self {
        self.evidence.push(citation);
        self
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let mut output = self.answer.clone();

        if !self.evidence.is_empty() {
            output.push_str("\n\nEvidence:");
            for cite in &self.evidence {
                output.push_str(&format!("\n  {}", cite.display()));
            }
        }

        if !self.sources.is_empty() {
            output.push_str("\n\nSources:");
            for cite in &self.sources {
                output.push_str(&format!("\n  {}", cite.display()));
            }
        }

        output
    }

    /// Has any citations?
    pub fn has_citations(&self) -> bool {
        !self.sources.is_empty() || !self.evidence.is_empty()
    }
}

/// Research result containing all fetched sources.
#[derive(Debug, Clone)]
pub struct ResearchResult {
    /// Original plan.
    pub plan: ResearchPlan,
    /// Fetched sources.
    pub sources: Vec<SourceContent>,
    /// Whether all required sources succeeded.
    pub all_required_succeeded: bool,
    /// Missing required sources.
    pub missing_required: Vec<String>,
}

impl ResearchResult {
    /// Create from plan and fetched sources.
    pub fn new(plan: ResearchPlan, sources: Vec<SourceContent>) -> Self {
        let required_ids: Vec<_> = plan
            .required_sources()
            .iter()
            .map(|s| s.id.clone())
            .collect();

        let mut missing = Vec::new();
        for id in &required_ids {
            let found = sources.iter().any(|s| s.request.id == *id && s.success);
            if !found {
                missing.push(id.clone());
            }
        }

        Self {
            plan,
            sources,
            all_required_succeeded: missing.is_empty(),
            missing_required: missing,
        }
    }

    /// Get successful sources.
    pub fn successful_sources(&self) -> Vec<&SourceContent> {
        self.sources.iter().filter(|s| s.success).collect()
    }

    /// Get source by ID.
    pub fn get_source(&self, id: &str) -> Option<&SourceContent> {
        self.sources.iter().find(|s| s.request.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_plan_from_intent() {
        let plan = ResearchPlan::from_intent("packages.update_system", "Update the system");
        assert!(!plan.sources.is_empty());
        assert!(plan
            .sources
            .iter()
            .any(|s| s.source_type == SourceType::Man));
    }

    #[test]
    fn test_citation_display() {
        let cite = Citation::source("S1", SourceType::Man, "pacman(8)", "SYSTEM UPGRADE");
        let display = cite.display();
        assert!(display.contains("[S1]"));
        assert!(display.contains("man pacman(8)"));
    }

    #[test]
    fn test_cited_answer() {
        let answer = CitedAnswer::new("Run pacman -Syu to update.", 0.9)
            .cite_source(Citation::source(
                "S1",
                SourceType::Man,
                "pacman(8)",
                "UPGRADE",
            ))
            .cite_evidence(Citation::evidence("E1", "pacman -Qu", "10 packages"));

        assert!(answer.has_citations());
        let display = answer.display();
        assert!(display.contains("Sources:"));
        assert!(display.contains("Evidence:"));
    }
}
