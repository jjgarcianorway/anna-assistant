//! Knowledge store for RAG-first answering (v0.0.32R).
//!
//! Provides fast local retrieval for:
//! - System facts (CPU, RAM, disk, packages)
//! - Arch Wiki/AUR metadata (cached locally)
//! - Learned recipes from resolved tickets
//!
//! Design principles:
//! - Offline-first: no web browsing at runtime unless configured
//! - Deterministic: stable ordering, reproducible results
//! - Auditable: full provenance, reversible (reset to wipe)
//! - Fast: keyword retrieval < 50ms typical
//!
//! Future: Embedding-based retrieval in Phase 7 (optional feature flag)

pub mod conversion;
pub mod index;
pub mod pack;
pub mod retrieval;
pub mod sources;
pub mod store;

// Re-export main types
pub use conversion::{recipe_to_knowledge_doc, should_convert_to_knowledge};
pub use index::KeywordIndex;
pub use pack::{get_builtin_docs, search_builtin_pack, try_builtin_answer, PackEntry, ARCH_PACK};
pub use retrieval::{RetrievalHit, RetrievalQuery};
pub use sources::{KnowledgeDoc, KnowledgeSource, Provenance};
pub use store::{KnowledgeStore, KnowledgeStoreTrait};

/// Performance budget: retrieval should be sub-50ms
pub const RETRIEVAL_BUDGET_MS: u64 = 50;

/// Default result limit for queries
pub const DEFAULT_QUERY_LIMIT: usize = 10;

/// Minimum confidence for high-quality answers
pub const HIGH_CONFIDENCE_THRESHOLD: u8 = 80;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_roundtrip() {
        let mut store = KnowledgeStore::new();

        // Create and insert a document
        let doc = KnowledgeDoc::new(
            KnowledgeSource::Recipe,
            "Install Vim",
            "To install vim on Arch Linux, run: pacman -S vim. This will install the vim editor.",
            vec!["vim".to_string(), "install".to_string()],
            Provenance::from_command("annad", "pacman -S vim", 100),
        );

        store.upsert(doc).unwrap();

        // Query for it
        let query = RetrievalQuery::new("install vim")
            .with_sources(vec![KnowledgeSource::Recipe])
            .with_limit(5);

        let hits = store.query(&query);

        assert_eq!(hits.len(), 1);
        assert!(hits[0].title.contains("Vim"));
        assert!(hits[0].snippet.contains("pacman"));
        assert_eq!(hits[0].confidence, 100);
    }

    #[test]
    fn test_multiple_sources() {
        let mut store = KnowledgeStore::new();

        // Add docs from different sources
        store.upsert(KnowledgeDoc::new(
            KnowledgeSource::SystemFact,
            "CPU Info",
            "AMD Ryzen 5 3600 6-core processor",
            vec!["cpu".to_string()],
            Provenance::from_command("annad", "lscpu", 100),
        )).unwrap();

        store.upsert(KnowledgeDoc::new(
            KnowledgeSource::PackageFact,
            "Installed Packages",
            "vim 9.0, neovim 0.9, emacs 28",
            vec!["packages".to_string()],
            Provenance::from_command("annad", "pacman -Q", 100),
        )).unwrap();

        store.upsert(KnowledgeDoc::new(
            KnowledgeSource::Recipe,
            "Configure Vim",
            "Edit ~/.vimrc to configure vim",
            vec!["vim".to_string()],
            Provenance::computed("annad", 90),
        )).unwrap();

        // Query all sources
        assert_eq!(store.len(), 3);

        // Query specific source
        let system_facts = store.query(
            &RetrievalQuery::new("processor")
                .with_sources(vec![KnowledgeSource::SystemFact])
        );
        assert_eq!(system_facts.len(), 1);
        assert_eq!(system_facts[0].source, KnowledgeSource::SystemFact);
    }
}
