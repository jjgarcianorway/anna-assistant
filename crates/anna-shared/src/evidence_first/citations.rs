//! Citations and evidence tracking (v0.0.435).
//!
//! Every claim must be backed by a citation to probe output or documentation.

use super::sources::KnowledgeSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique evidence identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceId(pub String);

impl EvidenceId {
    /// Create a new evidence ID.
    pub fn new(source_type: &str, name: &str) -> Self {
        Self(format!("{}:{}", source_type, name))
    }

    /// Create for a probe.
    pub fn probe(primitive_id: &str) -> Self {
        Self::new("probe", primitive_id)
    }

    /// Create for a man page.
    pub fn man(command: &str) -> Self {
        Self::new("man", command)
    }

    /// Create for help text.
    pub fn help(command: &str) -> Self {
        Self::new("help", command)
    }

    /// Create for wiki.
    pub fn wiki(page: &str) -> Self {
        Self::new("wiki", page)
    }

    /// Create for local docs.
    pub fn doc(path: &str) -> Self {
        Self::new("doc", path)
    }
}

impl std::fmt::Display for EvidenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A citation linking a claim to evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Evidence ID.
    pub evidence_id: EvidenceId,
    /// Source description (e.g., "man systemctl (OPTIONS)").
    pub source_label: String,
    /// Exact excerpt from source (max 1-2 lines).
    pub excerpt: String,
    /// Section or context within source.
    pub context: Option<String>,
    /// When this citation was created.
    pub created_at: u64,
}

impl Citation {
    /// Create a new citation.
    pub fn new(evidence_id: EvidenceId, source_label: &str, excerpt: &str) -> Self {
        Self {
            evidence_id,
            source_label: source_label.to_string(),
            excerpt: truncate_excerpt(excerpt),
            context: None,
            created_at: timestamp_now(),
        }
    }

    /// Add context.
    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    /// Format for display.
    pub fn format(&self) -> String {
        if let Some(ctx) = &self.context {
            format!(
                "• evidence: {} ({}) → \"{}\"",
                self.source_label, ctx, self.excerpt
            )
        } else {
            format!("• evidence: {} → \"{}\"", self.source_label, self.excerpt)
        }
    }

    /// Format as short reference.
    pub fn short_ref(&self) -> String {
        format!("[{}]", self.evidence_id)
    }
}

/// Store for all citations in a ticket.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CitationStore {
    /// All citations indexed by evidence ID.
    citations: HashMap<EvidenceId, Vec<Citation>>,
    /// Raw evidence content for audit.
    raw_evidence: HashMap<EvidenceId, RawEvidence>,
}

impl CitationStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add raw evidence.
    pub fn add_evidence(&mut self, id: EvidenceId, source: KnowledgeSource, content: &str) {
        self.raw_evidence.insert(
            id,
            RawEvidence {
                source,
                content: content.to_string(),
                retrieved_at: timestamp_now(),
            },
        );
    }

    /// Add a citation.
    pub fn add_citation(&mut self, citation: Citation) {
        self.citations
            .entry(citation.evidence_id.clone())
            .or_default()
            .push(citation);
    }

    /// Get all citations for an evidence ID.
    pub fn citations_for(&self, id: &EvidenceId) -> &[Citation] {
        self.citations.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get raw evidence.
    pub fn raw_evidence(&self, id: &EvidenceId) -> Option<&RawEvidence> {
        self.raw_evidence.get(id)
    }

    /// Get all evidence IDs.
    pub fn evidence_ids(&self) -> Vec<&EvidenceId> {
        self.raw_evidence.keys().collect()
    }

    /// Get all citations.
    pub fn all_citations(&self) -> Vec<&Citation> {
        self.citations.values().flatten().collect()
    }

    /// Count citations.
    pub fn citation_count(&self) -> usize {
        self.citations.values().map(|v| v.len()).sum()
    }

    /// Check if evidence exists.
    pub fn has_evidence(&self, id: &EvidenceId) -> bool {
        self.raw_evidence.contains_key(id)
    }

    /// Format all citations for display.
    pub fn format_citations(&self) -> String {
        let mut lines = Vec::new();
        for citation in self.all_citations() {
            lines.push(citation.format());
        }
        lines.join("\n")
    }

    /// Verify a citation exists in raw evidence.
    pub fn verify_citation(&self, citation: &Citation) -> bool {
        if let Some(raw) = self.raw_evidence.get(&citation.evidence_id) {
            // Check if excerpt appears in raw content (case-insensitive)
            raw.content
                .to_lowercase()
                .contains(&citation.excerpt.to_lowercase())
        } else {
            false
        }
    }
}

/// Raw evidence content for audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvidence {
    /// Source of evidence.
    pub source: KnowledgeSource,
    /// Full content.
    pub content: String,
    /// When retrieved.
    pub retrieved_at: u64,
}

impl RawEvidence {
    /// Truncate content for storage.
    pub fn truncated_content(&self, max_len: usize) -> &str {
        if self.content.len() > max_len {
            &self.content[..max_len]
        } else {
            &self.content
        }
    }
}

/// Truncate excerpt to max length.
fn truncate_excerpt(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() > super::MAX_CITATION_EXCERPT_LEN {
        format!("{}...", &trimmed[..super::MAX_CITATION_EXCERPT_LEN - 3])
    } else {
        trimmed.to_string()
    }
}

/// Get current timestamp.
fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_id_creation() {
        let probe = EvidenceId::probe("sys.boot.analyze");
        assert_eq!(probe.0, "probe:sys.boot.analyze");

        let man = EvidenceId::man("systemctl");
        assert_eq!(man.0, "man:systemctl");

        let wiki = EvidenceId::wiki("Systemd");
        assert_eq!(wiki.0, "wiki:Systemd");
    }

    #[test]
    fn test_citation_format() {
        let citation = Citation::new(
            EvidenceId::man("systemctl"),
            "man systemctl",
            "--failed: List failed units",
        )
        .with_context("OPTIONS");

        let formatted = citation.format();
        assert!(formatted.contains("man systemctl"));
        assert!(formatted.contains("OPTIONS"));
        assert!(formatted.contains("--failed"));
    }

    #[test]
    fn test_citation_store() {
        let mut store = CitationStore::new();

        let id = EvidenceId::probe("sys.mem.free");
        store.add_evidence(
            id.clone(),
            KnowledgeSource::ProbeOutput("sys.mem.free".to_string()),
            "MemTotal: 32000000 kB\nMemFree: 16000000 kB",
        );

        store.add_citation(Citation::new(
            id.clone(),
            "probe:sys.mem.free",
            "MemFree: 16000000 kB",
        ));

        assert!(store.has_evidence(&id));
        assert_eq!(store.citation_count(), 1);
    }

    #[test]
    fn test_verify_citation() {
        let mut store = CitationStore::new();

        let id = EvidenceId::man("ls");
        store.add_evidence(
            id.clone(),
            KnowledgeSource::ManPage(super::super::sources::ManPageSource::new("ls")),
            "list directory contents",
        );

        // Valid citation
        let valid = Citation::new(id.clone(), "man ls", "list directory");
        assert!(store.verify_citation(&valid));

        // Invalid citation (not in content)
        let invalid = Citation::new(id.clone(), "man ls", "something not there");
        assert!(!store.verify_citation(&invalid));
    }

    #[test]
    fn test_truncate_excerpt() {
        let short = "short excerpt";
        assert_eq!(truncate_excerpt(short), short);

        let long = "a".repeat(300);
        let truncated = truncate_excerpt(&long);
        assert!(truncated.len() <= super::super::MAX_CITATION_EXCERPT_LEN);
        assert!(truncated.ends_with("..."));
    }
}
