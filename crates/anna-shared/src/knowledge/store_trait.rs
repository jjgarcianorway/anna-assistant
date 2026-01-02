//! Knowledge store trait definition.

use super::retrieval::{RetrievalHit, RetrievalQuery};
use super::sources::KnowledgeDoc;

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
