//! Evidence bundle and metadata

use serde::{Deserialize, Serialize};

use super::evidence::{DocSnippet, ProbeEvidence, RecipeMatch};
use super::utils::current_millis;

/// Complete evidence bundle for specialist consumption
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// Probe results (system state facts)
    pub probes: Vec<ProbeEvidence>,
    /// Documentation snippets (authoritative text)
    pub docs: Vec<DocSnippet>,
    /// Matching recipe candidates (learned patterns)
    pub recipes: Vec<RecipeMatch>,
    /// Bundle metadata
    pub metadata: BundleMetadata,
}

impl EvidenceBundle {
    pub fn new(ticket_id: &str) -> Self {
        Self {
            probes: vec![],
            docs: vec![],
            recipes: vec![],
            metadata: BundleMetadata::new(ticket_id),
        }
    }

    /// Check if bundle has any useful evidence
    pub fn has_evidence(&self) -> bool {
        !self.probes.is_empty() || !self.docs.is_empty()
    }

    /// Get total evidence count
    pub fn evidence_count(&self) -> usize {
        self.probes.len() + self.docs.len() + self.recipes.len()
    }

    /// Add a probe result
    pub fn add_probe(&mut self, probe: ProbeEvidence) {
        self.probes.push(probe);
    }

    /// Add a doc snippet
    pub fn add_doc(&mut self, doc: DocSnippet) {
        self.docs.push(doc);
    }

    /// Add a recipe match
    pub fn add_recipe(&mut self, recipe: RecipeMatch) {
        self.recipes.push(recipe);
    }

    /// Get all evidence IDs for citation
    pub fn all_evidence_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.probes.iter().map(|p| p.id.clone()).collect();
        ids.extend(self.docs.iter().map(|d| d.id.clone()));
        ids.extend(self.recipes.iter().map(|r| r.id.clone()));
        ids
    }

    /// Format for specialist context (concise)
    pub fn format_for_specialist(&self) -> String {
        let mut output = String::new();

        if !self.probes.is_empty() {
            output.push_str("=== PROBE EVIDENCE ===\n");
            for probe in &self.probes {
                output.push_str(&format!(
                    "[{}] {}\n{}\n\n",
                    probe.id, probe.summary, probe.excerpt
                ));
            }
        }

        if !self.docs.is_empty() {
            output.push_str("=== DOCUMENTATION ===\n");
            for doc in &self.docs {
                output.push_str(&format!(
                    "[{}] {} ({})\n{}\n\n",
                    doc.id, doc.title, doc.source, doc.snippet
                ));
            }
        }

        if !self.recipes.is_empty() {
            output.push_str("=== MATCHING RECIPES ===\n");
            for recipe in &self.recipes {
                output.push_str(&format!(
                    "[{}] {} (confidence: {}%)\n{}\n\n",
                    recipe.id, recipe.title, recipe.confidence, recipe.summary
                ));
            }
        }

        if output.is_empty() {
            output.push_str("No evidence gathered.\n");
        }

        output
    }
}

/// Bundle metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// Ticket ID
    pub ticket_id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Time spent gathering evidence (ms)
    pub gather_time_ms: u64,
    /// Probes that were run
    pub probes_run: Vec<String>,
    /// Doc sources searched
    pub docs_searched: Vec<String>,
}

impl BundleMetadata {
    pub fn new(ticket_id: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            created_at: current_millis(),
            gather_time_ms: 0,
            probes_run: vec![],
            docs_searched: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_bundle() {
        let mut bundle = EvidenceBundle::new("TEST-001");
        assert!(!bundle.has_evidence());

        bundle.add_probe(ProbeEvidence::new(
            "probe:df_root",
            "df -h /",
            "Root filesystem 75% full",
            "/dev/sda1 100G 75G 25G 75% /",
        ));

        assert!(bundle.has_evidence());
        assert_eq!(bundle.evidence_count(), 1);
    }
}
