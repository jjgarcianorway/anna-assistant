//! Provenance Tracking - Every answer must cite its sources.
//!
//! Any answer or action that cites memory must carry provenance pointers:
//! - Which episode
//! - Which probe output
//! - Which doc chunk
//!
//! This is mandatory for accountability and debugging.

use serde::{Deserialize, Serialize};

/// A provenance record tracking where knowledge came from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    /// The source of this knowledge
    pub source: ProvenanceSource,
    /// Confidence at retrieval time
    pub confidence: f32,
    /// When this was retrieved
    pub retrieved_at: String,
    /// What this knowledge was used for (set after use)
    pub used_for: Option<String>,
}

/// The specific source of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvenanceSource {
    /// From a live system probe
    LiveProbe {
        /// The command that was run
        command: String,
        /// When it was run
        timestamp: String,
    },
    /// From trusted documentation
    TrustedDocs {
        /// Article title
        article: String,
        /// Section within article
        section: Option<String>,
        /// Source URL
        url: String,
    },
    /// From a validated skill
    ValidatedSkill {
        /// Skill/recipe ID
        skill_id: String,
        /// Skill name
        skill_name: String,
        /// Trust tier
        tier: String,
    },
    /// From semantic memory (patterns)
    SemanticMemory {
        /// Pattern ID
        pattern_id: String,
        /// Keywords that matched
        keywords: Vec<String>,
        /// Evidence count
        evidence_count: u32,
    },
    /// From episodic memory (experiences)
    EpisodicMemory {
        /// Experience ID
        experience_id: String,
        /// Original question
        original_question: String,
        /// When it was created
        created_at: String,
    },
    /// From man page
    ManPage {
        /// Command name
        command: String,
        /// Section (1-8)
        section: u8,
    },
    /// From --help output
    HelpOutput {
        /// Command name
        command: String,
        /// Timestamp
        timestamp: String,
    },
    /// Combined from multiple sources
    Combined {
        /// Source IDs
        sources: Vec<String>,
    },
}

impl ProvenanceSource {
    /// Get a human-readable citation string
    pub fn citation(&self) -> String {
        match self {
            ProvenanceSource::LiveProbe { command, timestamp } => {
                format!("[Probe: `{}` at {}]", command, timestamp)
            }
            ProvenanceSource::TrustedDocs { article, section, url } => {
                if let Some(sec) = section {
                    format!("[Arch Wiki: {} - {}]({})", article, sec, url)
                } else {
                    format!("[Arch Wiki: {}]({})", article, url)
                }
            }
            ProvenanceSource::ValidatedSkill { skill_name, tier, .. } => {
                format!("[Skill: {} ({})]", skill_name, tier)
            }
            ProvenanceSource::SemanticMemory { keywords, evidence_count, .. } => {
                format!(
                    "[Pattern: {} ({} evidence)]",
                    keywords.join(", "),
                    evidence_count
                )
            }
            ProvenanceSource::EpisodicMemory { original_question, .. } => {
                format!("[Past experience: \"{}\"]", truncate(original_question, 50))
            }
            ProvenanceSource::ManPage { command, section } => {
                format!("[man {}({})]", command, section)
            }
            ProvenanceSource::HelpOutput { command, .. } => {
                format!("[{} --help]", command)
            }
            ProvenanceSource::Combined { sources } => {
                format!("[Combined: {} sources]", sources.len())
            }
        }
    }

    /// Get source type name
    pub fn source_type(&self) -> &'static str {
        match self {
            ProvenanceSource::LiveProbe { .. } => "live_probe",
            ProvenanceSource::TrustedDocs { .. } => "trusted_docs",
            ProvenanceSource::ValidatedSkill { .. } => "validated_skill",
            ProvenanceSource::SemanticMemory { .. } => "semantic_memory",
            ProvenanceSource::EpisodicMemory { .. } => "episodic_memory",
            ProvenanceSource::ManPage { .. } => "man_page",
            ProvenanceSource::HelpOutput { .. } => "help_output",
            ProvenanceSource::Combined { .. } => "combined",
        }
    }

    /// Is this a ground truth source?
    pub fn is_ground_truth(&self) -> bool {
        matches!(
            self,
            ProvenanceSource::LiveProbe { .. }
                | ProvenanceSource::TrustedDocs { .. }
                | ProvenanceSource::ManPage { .. }
                | ProvenanceSource::HelpOutput { .. }
        )
    }
}

impl ProvenanceRecord {
    /// Create a new provenance record
    pub fn new(source: ProvenanceSource, confidence: f32) -> Self {
        Self {
            source,
            confidence,
            retrieved_at: chrono::Utc::now().to_rfc3339(),
            used_for: None,
        }
    }

    /// Mark this provenance as used for something
    pub fn mark_used(&mut self, purpose: &str) {
        self.used_for = Some(purpose.to_string());
    }

    /// Get citation for display
    pub fn citation(&self) -> String {
        self.source.citation()
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Collection of provenance records for an answer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// All provenance records
    pub records: Vec<ProvenanceRecord>,
    /// Summary of sources used
    pub summary: String,
}

impl ProvenanceChain {
    /// Create empty chain
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a record
    pub fn add(&mut self, record: ProvenanceRecord) {
        self.records.push(record);
        self.update_summary();
    }

    /// Update summary string
    fn update_summary(&mut self) {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for r in &self.records {
            *counts.entry(r.source.source_type()).or_insert(0) += 1;
        }

        let parts: Vec<String> = counts
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();

        self.summary = parts.join(", ");
    }

    /// Get all citations
    pub fn citations(&self) -> Vec<String> {
        self.records.iter().map(|r| r.citation()).collect()
    }

    /// Format for display
    pub fn format_citations(&self) -> String {
        let citations = self.citations();
        if citations.is_empty() {
            return "No sources cited.".to_string();
        }

        format!("Sources:\n{}", citations.join("\n"))
    }

    /// Check if chain includes ground truth
    pub fn has_ground_truth(&self) -> bool {
        self.records.iter().any(|r| r.source.is_ground_truth())
    }

    /// Get confidence based on sources
    pub fn overall_confidence(&self) -> f32 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f32 = self.records.iter().map(|r| r.confidence).sum();
        total / self.records.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_format() {
        let source = ProvenanceSource::TrustedDocs {
            article: "Pacman".to_string(),
            section: Some("Installing packages".to_string()),
            url: "https://wiki.archlinux.org/title/Pacman".to_string(),
        };

        let citation = source.citation();
        assert!(citation.contains("Pacman"));
        assert!(citation.contains("Installing packages"));
    }

    #[test]
    fn test_episodic_not_ground_truth() {
        let source = ProvenanceSource::EpisodicMemory {
            experience_id: "123".to_string(),
            original_question: "test".to_string(),
            created_at: "2024-01-01".to_string(),
        };

        assert!(!source.is_ground_truth());
    }

    #[test]
    fn test_provenance_chain() {
        let mut chain = ProvenanceChain::new();

        chain.add(ProvenanceRecord::new(
            ProvenanceSource::LiveProbe {
                command: "ls".to_string(),
                timestamp: "now".to_string(),
            },
            1.0,
        ));

        assert!(chain.has_ground_truth());
        assert!(!chain.citations().is_empty());
    }
}
