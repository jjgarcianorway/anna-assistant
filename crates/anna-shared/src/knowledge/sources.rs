//! Knowledge source types and data model (v0.0.32R).
//!
//! Provenance-rich documents for the RAG knowledge store.

use serde::{Deserialize, Serialize};

/// Source of a knowledge document
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSource {
    /// Learned recipe from resolved tickets
    Recipe,
    /// System fact (CPU, RAM, disk, etc.)
    SystemFact,
    /// Package installation fact
    PackageFact,
    /// Arch Wiki article (cached locally)
    ArchWiki,
    /// AUR package metadata
    AUR,
    /// Journal/log entry
    Journal,
    /// Usage telemetry (future)
    Usage,
    /// Built-in static knowledge (v0.0.39, never expires)
    BuiltIn,
}

impl std::fmt::Display for KnowledgeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recipe => write!(f, "recipe"),
            Self::SystemFact => write!(f, "system_fact"),
            Self::PackageFact => write!(f, "package_fact"),
            Self::ArchWiki => write!(f, "arch_wiki"),
            Self::AUR => write!(f, "aur"),
            Self::Journal => write!(f, "journal"),
            Self::Usage => write!(f, "usage"),
            Self::BuiltIn => write!(f, "built_in"),
        }
    }
}

/// Provenance information for audit and reversibility
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Provenance {
    /// Who/what collected this document
    pub collected_by: String,
    /// Command used to collect (if applicable)
    pub command: Option<String>,
    /// File path source (if applicable)
    pub path: Option<String>,
    /// Confidence in document correctness (0-100)
    pub confidence: u8,
}

impl Provenance {
    /// Create provenance from a command
    pub fn from_command(collector: &str, cmd: &str, confidence: u8) -> Self {
        Self {
            collected_by: collector.to_string(),
            command: Some(cmd.to_string()),
            path: None,
            confidence,
        }
    }

    /// Create provenance from a file path
    pub fn from_path(collector: &str, path: &str, confidence: u8) -> Self {
        Self {
            collected_by: collector.to_string(),
            command: None,
            path: Some(path.to_string()),
            confidence,
        }
    }

    /// Create provenance for derived/computed data
    pub fn computed(collector: &str, confidence: u8) -> Self {
        Self {
            collected_by: collector.to_string(),
            command: None,
            path: None,
            confidence,
        }
    }
}

/// A knowledge document with full provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDoc {
    /// Stable hash key (deterministic from content)
    pub id: String,
    /// Source of this document
    pub source: KnowledgeSource,
    /// Document title (for display)
    pub title: String,
    /// Document body (plain text, searchable)
    pub body: String,
    /// Tags for filtering (stable ordering)
    pub tags: Vec<String>,
    /// Monotonic sequence number at creation
    pub created_seq: u64,
    /// Monotonic sequence number at last update
    pub updated_seq: u64,
    /// TTL hint in days (for refresh decisions)
    pub ttl_hint: Option<u64>,
    /// Full provenance for audit
    pub provenance: Provenance,
}

impl KnowledgeDoc {
    /// Create a new knowledge document with computed ID
    pub fn new(
        source: KnowledgeSource,
        title: impl Into<String>,
        body: impl Into<String>,
        tags: Vec<String>,
        provenance: Provenance,
    ) -> Self {
        let title = title.into();
        let body = body.into();
        let id = compute_doc_id(&source, &title, &body);
        Self {
            id,
            source,
            title,
            body,
            tags,
            created_seq: 0,
            updated_seq: 0,
            ttl_hint: None,
            provenance,
        }
    }

    /// Create with specific ID (for deserialization)
    pub fn with_id(
        id: String,
        source: KnowledgeSource,
        title: String,
        body: String,
        tags: Vec<String>,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            source,
            title,
            body,
            tags,
            created_seq: 0,
            updated_seq: 0,
            ttl_hint: None,
            provenance,
        }
    }

    /// Set TTL hint
    pub fn with_ttl(mut self, days: u64) -> Self {
        self.ttl_hint = Some(days);
        self
    }

    /// Set sequence numbers
    pub fn with_seq(mut self, created: u64, updated: u64) -> Self {
        self.created_seq = created;
        self.updated_seq = updated;
        self
    }

    /// Check if document is high-confidence (>= 80)
    pub fn is_high_confidence(&self) -> bool {
        self.provenance.confidence >= 80
    }

    /// Get all searchable text (title + body)
    pub fn searchable_text(&self) -> String {
        format!("{} {}", self.title, self.body)
    }
}

/// Compute deterministic document ID from source, title, and body
fn compute_doc_id(source: &KnowledgeSource, title: &str, body: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    source.to_string().hash(&mut hasher);
    title.hash(&mut hasher);
    // Only hash first 256 chars of body for stability during updates
    body.chars().take(256).collect::<String>().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_id_deterministic() {
        let doc1 = KnowledgeDoc::new(
            KnowledgeSource::SystemFact,
            "CPU Info",
            "AMD Ryzen 5 3600",
            vec!["cpu".to_string()],
            Provenance::from_command("annad", "lscpu", 100),
        );
        let doc2 = KnowledgeDoc::new(
            KnowledgeSource::SystemFact,
            "CPU Info",
            "AMD Ryzen 5 3600",
            vec!["cpu".to_string()],
            Provenance::from_command("annad", "lscpu", 100),
        );
        assert_eq!(doc1.id, doc2.id);
    }

    #[test]
    fn test_doc_id_different_for_different_content() {
        let doc1 = KnowledgeDoc::new(
            KnowledgeSource::SystemFact,
            "CPU Info",
            "AMD Ryzen 5 3600",
            vec![],
            Provenance::computed("test", 100),
        );
        let doc2 = KnowledgeDoc::new(
            KnowledgeSource::SystemFact,
            "CPU Info",
            "Intel Core i7",
            vec![],
            Provenance::computed("test", 100),
        );
        assert_ne!(doc1.id, doc2.id);
    }

    #[test]
    fn test_provenance_from_command() {
        let p = Provenance::from_command("annad", "pacman -Q", 90);
        assert_eq!(p.collected_by, "annad");
        assert_eq!(p.command, Some("pacman -Q".to_string()));
        assert_eq!(p.confidence, 90);
    }

    #[test]
    fn test_high_confidence() {
        let doc = KnowledgeDoc::new(
            KnowledgeSource::Recipe,
            "Test",
            "Body",
            vec![],
            Provenance::computed("test", 80),
        );
        assert!(doc.is_high_confidence());

        let doc2 = KnowledgeDoc::new(
            KnowledgeSource::Recipe,
            "Test",
            "Body",
            vec![],
            Provenance::computed("test", 79),
        );
        assert!(!doc2.is_high_confidence());
    }
}
