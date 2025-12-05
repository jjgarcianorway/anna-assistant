//! Keyword-based index for fast retrieval (v0.0.32R).
//!
//! Simple inverted index with BM25-lite scoring. Deterministic and fast.
//! Embedding-based retrieval is Phase 7 (optional feature flag).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Tokenize text into searchable tokens (deterministic)
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|s| s.len() >= 2) // Skip single chars
        .map(String::from)
        .collect()
}

/// Unique tokens from text (for document indexing)
pub fn unique_tokens(text: &str) -> HashSet<String> {
    tokenize(text).into_iter().collect()
}

/// Inverted index entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostingList {
    /// Document IDs containing this token
    pub doc_ids: Vec<String>,
    /// Term frequency in each document (parallel to doc_ids)
    pub term_freqs: Vec<u32>,
}

/// Keyword index for fast retrieval
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeywordIndex {
    /// Token -> posting list
    pub index: HashMap<String, PostingList>,
    /// Document lengths (for BM25 normalization)
    pub doc_lengths: HashMap<String, u32>,
    /// Average document length
    pub avg_doc_length: f32,
    /// Total number of documents
    pub doc_count: u32,
}

impl KeywordIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self::default()
    }

    /// Index a document
    pub fn add_document(&mut self, doc_id: &str, text: &str) {
        let tokens = tokenize(text);
        let doc_length = tokens.len() as u32;

        // Update doc length
        self.doc_lengths.insert(doc_id.to_string(), doc_length);
        self.doc_count = self.doc_lengths.len() as u32;

        // Recalculate average
        let total_len: u32 = self.doc_lengths.values().sum();
        self.avg_doc_length = if self.doc_count > 0 {
            total_len as f32 / self.doc_count as f32
        } else {
            0.0
        };

        // Count term frequencies
        let mut term_counts: HashMap<String, u32> = HashMap::new();
        for token in tokens {
            *term_counts.entry(token).or_insert(0) += 1;
        }

        // Update inverted index
        for (token, freq) in term_counts {
            let posting = self.index.entry(token).or_default();

            // Check if doc already indexed (update case)
            if let Some(pos) = posting.doc_ids.iter().position(|id| id == doc_id) {
                posting.term_freqs[pos] = freq;
            } else {
                posting.doc_ids.push(doc_id.to_string());
                posting.term_freqs.push(freq);
            }
        }
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: &str) {
        // Remove from doc lengths
        self.doc_lengths.remove(doc_id);
        self.doc_count = self.doc_lengths.len() as u32;

        // Recalculate average
        let total_len: u32 = self.doc_lengths.values().sum();
        self.avg_doc_length = if self.doc_count > 0 {
            total_len as f32 / self.doc_count as f32
        } else {
            0.0
        };

        // Remove from posting lists
        for posting in self.index.values_mut() {
            if let Some(pos) = posting.doc_ids.iter().position(|id| id == doc_id) {
                posting.doc_ids.remove(pos);
                posting.term_freqs.remove(pos);
            }
        }

        // Clean up empty posting lists
        self.index.retain(|_, v| !v.doc_ids.is_empty());
    }

    /// Search the index and return scored document IDs
    /// Uses BM25-lite scoring (simplified, deterministic)
    pub fn search(&self, query: &str, limit: usize) -> Vec<(String, i32)> {
        let query_tokens = tokenize(query);
        if query_tokens.is_empty() {
            return vec![];
        }

        // Score each document
        let mut scores: HashMap<String, f32> = HashMap::new();

        for token in &query_tokens {
            if let Some(posting) = self.index.get(token) {
                // IDF: log((N - n + 0.5) / (n + 0.5))
                let n = posting.doc_ids.len() as f32;
                let idf = ((self.doc_count as f32 - n + 0.5) / (n + 0.5) + 1.0).ln();

                for (doc_id, &tf) in posting.doc_ids.iter().zip(posting.term_freqs.iter()) {
                    let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&1) as f32;

                    // BM25 scoring: IDF * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * dl/avgdl))
                    const K1: f32 = 1.2;
                    const B: f32 = 0.75;

                    let norm = 1.0 - B + B * (doc_len / self.avg_doc_length.max(1.0));
                    let tf_score = (tf as f32 * (K1 + 1.0)) / (tf as f32 + K1 * norm);
                    let score = idf * tf_score;

                    *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                }
            }
        }

        // Convert to integer scores and sort deterministically
        let mut results: Vec<(String, i32)> = scores
            .into_iter()
            .map(|(id, score)| (id, (score * 1000.0) as i32))
            .collect();

        // Deterministic sort: score desc, then id asc
        results.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
        });

        results.truncate(limit);
        results
    }

    /// Get document count
    pub fn len(&self) -> usize {
        self.doc_count as usize
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.doc_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello World! This is a test_token and foo-bar.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test_token".to_string()));
        assert!(tokens.contains(&"foo-bar".to_string()));
        // Single char 'a' should be filtered
        assert!(!tokens.contains(&"a".to_string()));
    }

    #[test]
    fn test_index_and_search() {
        let mut index = KeywordIndex::new();
        index.add_document("doc1", "how to install vim editor");
        index.add_document("doc2", "how to configure neovim");
        index.add_document("doc3", "pacman package manager");

        let results = index.search("vim", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "doc1");

        let results = index.search("how to", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_deterministic_ordering() {
        let mut index = KeywordIndex::new();
        index.add_document("aaa", "test document one");
        index.add_document("bbb", "test document two");
        index.add_document("ccc", "test document three");

        // Same score should sort by ID
        let results1 = index.search("test document", 10);
        let results2 = index.search("test document", 10);

        assert_eq!(results1, results2);
        // With same scores, should be sorted by id asc
        assert!(results1.iter().map(|(id, _)| id).collect::<Vec<_>>()
            .windows(2)
            .all(|w| w[0] <= w[1] || results1.iter().find(|(id, _)| id == w[0]).unwrap().1
                > results1.iter().find(|(id, _)| id == w[1]).unwrap().1));
    }

    #[test]
    fn test_remove_document() {
        let mut index = KeywordIndex::new();
        index.add_document("doc1", "vim editor");
        index.add_document("doc2", "vim configuration");

        assert_eq!(index.search("vim", 10).len(), 2);

        index.remove_document("doc1");

        assert_eq!(index.search("vim", 10).len(), 1);
        assert_eq!(index.search("vim", 10)[0].0, "doc2");
    }

    #[test]
    fn test_update_document() {
        let mut index = KeywordIndex::new();
        index.add_document("doc1", "old content about vim");

        let results = index.search("vim", 10);
        assert_eq!(results.len(), 1);

        // Update same doc
        index.add_document("doc1", "new content about neovim");

        // Should find neovim now
        let results = index.search("neovim", 10);
        assert_eq!(results.len(), 1);
    }

    // Golden test for deterministic search
    #[test]
    fn golden_search_ordering() {
        let mut index = KeywordIndex::new();
        index.add_document("d1", "arch linux installation guide");
        index.add_document("d2", "arch linux pacman usage");
        index.add_document("d3", "debian linux guide");

        let results = index.search("arch linux", 10);
        // d1 and d2 have "arch linux", d3 only has "linux"
        assert!(results.len() >= 2);
        // arch linux docs should score higher
        assert!(results[0].0 == "d1" || results[0].0 == "d2");
    }
}
