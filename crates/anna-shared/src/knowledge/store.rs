//! Knowledge store implementation (v0.0.32R).
//!
//! Persistent document store with keyword indexing.
//! Location: ~/.anna/knowledge/

use super::index::KeywordIndex;
use super::retrieval::{doc_matches_filters, RetrievalHit, RetrievalQuery};
use super::sources::KnowledgeDoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;

/// Knowledge store trait for pluggable implementations
pub trait KnowledgeStoreTrait {
    /// Insert or update a document
    fn upsert(&mut self, doc: KnowledgeDoc) -> Result<(), String>;
    /// Query for documents
    fn query(&self, q: &RetrievalQuery) -> Vec<RetrievalHit>;
    /// Get a document by ID
    fn get(&self, id: &str) -> Option<&KnowledgeDoc>;
    /// Remove a document
    fn remove(&mut self, id: &str) -> Option<KnowledgeDoc>;
    /// Get document count
    fn len(&self) -> usize;
    /// Check if empty
    fn is_empty(&self) -> bool;
}

/// File-based knowledge store with keyword index
#[derive(Debug, Default)]
pub struct KnowledgeStore {
    /// Documents by ID
    docs: HashMap<String, KnowledgeDoc>,
    /// Keyword index
    index: KeywordIndex,
    /// Monotonic sequence counter
    seq: u64,
    /// Store directory path
    path: PathBuf,
    /// Dirty flag for persistence
    dirty: bool,
}

/// Wire format for store metadata
#[derive(Debug, Serialize, Deserialize)]
struct StoreMetadata {
    version: u32,
    seq: u64,
    doc_count: usize,
}

impl KnowledgeStore {
    /// Default store path
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".anna")
            .join("knowledge")
    }

    /// Create a new store at the default path
    pub fn new() -> Self {
        Self::at_path(Self::default_path())
    }

    /// Create a new store at a specific path
    pub fn at_path(path: PathBuf) -> Self {
        Self {
            docs: HashMap::new(),
            index: KeywordIndex::new(),
            seq: 0,
            path,
            dirty: false,
        }
    }

    /// Load store from disk
    pub fn load() -> Self {
        Self::load_from_path(&Self::default_path())
    }

    /// Load store from specific path
    pub fn load_from_path(path: &PathBuf) -> Self {
        let mut store = Self::at_path(path.clone());

        // Load docs from JSONL file
        let docs_path = path.join("docs.jsonl");
        if docs_path.exists() {
            if let Ok(file) = fs::File::open(&docs_path) {
                let reader = std::io::BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    if let Ok(doc) = serde_json::from_str::<KnowledgeDoc>(&line) {
                        // Update seq counter
                        store.seq = store.seq.max(doc.updated_seq);
                        // Index the document
                        store.index.add_document(&doc.id, &doc.searchable_text());
                        store.docs.insert(doc.id.clone(), doc);
                    }
                }
            }
        }

        // Load metadata
        let meta_path = path.join("meta.json");
        if meta_path.exists() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<StoreMetadata>(&content) {
                    store.seq = store.seq.max(meta.seq);
                }
            }
        }

        store.dirty = false;
        store
    }

    /// Save store to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        self.save_to_path(&self.path)
    }

    /// Save store to specific path
    pub fn save_to_path(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        fs::create_dir_all(path)?;

        // Write docs to JSONL (sorted for determinism)
        let docs_path = path.join("docs.jsonl");
        let file = fs::File::create(&docs_path)?;
        let mut writer = BufWriter::new(file);

        let mut docs: Vec<_> = self.docs.values().collect();
        docs.sort_by_key(|d| &d.id);

        for doc in docs {
            let line = serde_json::to_string(doc)?;
            writeln!(writer, "{}", line)?;
        }
        writer.flush()?;

        // Write index
        let index_path = path.join("index.json");
        let index_json = serde_json::to_string_pretty(&self.index)?;
        fs::write(&index_path, index_json)?;

        // Write metadata
        let meta_path = path.join("meta.json");
        let meta = StoreMetadata {
            version: 1,
            seq: self.seq,
            doc_count: self.docs.len(),
        };
        let meta_json = serde_json::to_string_pretty(&meta)?;
        fs::write(&meta_path, meta_json)?;

        Ok(())
    }

    /// Get next sequence number
    fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    /// Check if store has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Reset store (clear all documents)
    pub fn reset(&mut self) {
        self.docs.clear();
        self.index = KeywordIndex::new();
        self.seq = 0;
        self.dirty = true;
    }

    /// Get all documents
    pub fn all_docs(&self) -> Vec<&KnowledgeDoc> {
        self.docs.values().collect()
    }

    /// Get documents by source
    pub fn docs_by_source(&self, source: &super::sources::KnowledgeSource) -> Vec<&KnowledgeDoc> {
        self.docs.values().filter(|d| &d.source == source).collect()
    }
}

impl KnowledgeStoreTrait for KnowledgeStore {
    fn upsert(&mut self, mut doc: KnowledgeDoc) -> Result<(), String> {
        let seq = self.next_seq();

        // Check if update or insert
        if let Some(existing) = self.docs.get(&doc.id) {
            doc.created_seq = existing.created_seq;
            doc.updated_seq = seq;
        } else {
            doc.created_seq = seq;
            doc.updated_seq = seq;
        }

        // Update index
        self.index.add_document(&doc.id, &doc.searchable_text());

        // Store document
        self.docs.insert(doc.id.clone(), doc);
        self.dirty = true;

        Ok(())
    }

    fn query(&self, q: &RetrievalQuery) -> Vec<RetrievalHit> {
        // Get keyword search results
        let keyword_hits = self.index.search(&q.text, q.limit * 2); // Get extra for filtering

        // Filter and convert to hits
        let mut hits: Vec<RetrievalHit> = keyword_hits
            .into_iter()
            .filter_map(|(id, score)| {
                let doc = self.docs.get(&id)?;
                if doc_matches_filters(doc, q) {
                    Some(RetrievalHit::from_doc(doc, score, &q.text))
                } else {
                    None
                }
            })
            .collect();

        // Ensure deterministic ordering (already sorted by index, but re-sort for safety)
        hits.sort_by(|a, b| {
            b.score.cmp(&a.score).then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        hits.truncate(q.limit);
        hits
    }

    fn get(&self, id: &str) -> Option<&KnowledgeDoc> {
        self.docs.get(id)
    }

    fn remove(&mut self, id: &str) -> Option<KnowledgeDoc> {
        if let Some(doc) = self.docs.remove(id) {
            self.index.remove_document(id);
            self.dirty = true;
            Some(doc)
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.docs.len()
    }

    fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::sources::{KnowledgeSource, Provenance};
    use tempfile::TempDir;

    fn test_doc(title: &str, body: &str) -> KnowledgeDoc {
        KnowledgeDoc::new(
            KnowledgeSource::Recipe,
            title,
            body,
            vec!["test".to_string()],
            Provenance::computed("test", 100),
        )
    }

    #[test]
    fn test_upsert_and_query() {
        let mut store = KnowledgeStore::new();

        let doc = test_doc("Vim Configuration", "How to configure vim editor with plugins");
        store.upsert(doc).unwrap();

        let results = store.query(&RetrievalQuery::new("vim").with_limit(10));
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Vim"));
    }

    #[test]
    fn test_query_filters() {
        let mut store = KnowledgeStore::new();

        store.upsert(KnowledgeDoc::new(
            KnowledgeSource::Recipe,
            "Recipe Doc",
            "vim recipe",
            vec!["vim".to_string()],
            Provenance::computed("test", 100),
        )).unwrap();

        store.upsert(KnowledgeDoc::new(
            KnowledgeSource::ArchWiki,
            "Wiki Doc",
            "vim wiki",
            vec!["vim".to_string()],
            Provenance::computed("test", 100),
        )).unwrap();

        // Query all
        let all = store.query(&RetrievalQuery::new("vim").with_limit(10));
        assert_eq!(all.len(), 2);

        // Query only recipes
        let recipes = store.query(
            &RetrievalQuery::new("vim")
                .with_sources(vec![KnowledgeSource::Recipe])
                .with_limit(10)
        );
        assert_eq!(recipes.len(), 1);
        assert_eq!(recipes[0].source, KnowledgeSource::Recipe);
    }

    #[test]
    fn test_persistence() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        // Create and save
        {
            let mut store = KnowledgeStore::at_path(path.clone());
            store.upsert(test_doc("Test Doc", "test content")).unwrap();
            store.save().unwrap();
        }

        // Load and verify
        {
            let store = KnowledgeStore::load_from_path(&path);
            assert_eq!(store.len(), 1);
            let results = store.query(&RetrievalQuery::new("test").with_limit(10));
            assert_eq!(results.len(), 1);
        }
    }

    #[test]
    fn test_remove() {
        let mut store = KnowledgeStore::new();
        let doc = test_doc("Remove Me", "content to remove");
        let id = doc.id.clone();

        store.upsert(doc).unwrap();
        assert_eq!(store.len(), 1);

        store.remove(&id);
        assert_eq!(store.len(), 0);
        assert!(store.get(&id).is_none());
    }

    #[test]
    fn test_reset() {
        let mut store = KnowledgeStore::new();
        store.upsert(test_doc("Doc 1", "content")).unwrap();
        store.upsert(test_doc("Doc 2", "content")).unwrap();

        assert_eq!(store.len(), 2);

        store.reset();

        assert_eq!(store.len(), 0);
        assert!(store.is_dirty());
    }

    // Golden test for deterministic query results
    #[test]
    fn golden_query_ordering() {
        let mut store = KnowledgeStore::new();

        store.upsert(test_doc("aaa", "vim editor guide")).unwrap();
        store.upsert(test_doc("bbb", "vim configuration")).unwrap();
        store.upsert(test_doc("ccc", "vim plugins")).unwrap();

        let results1 = store.query(&RetrievalQuery::new("vim").with_limit(10));
        let results2 = store.query(&RetrievalQuery::new("vim").with_limit(10));

        // Same query should always return same order
        let ids1: Vec<_> = results1.iter().map(|h| &h.doc_id).collect();
        let ids2: Vec<_> = results2.iter().map(|h| &h.doc_id).collect();
        assert_eq!(ids1, ids2);
    }
}
