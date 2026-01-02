//! The knowledge index store.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

use super::doc_cache::CachedDoc;
use super::fact::LearnedFact;
use super::pattern::LearnedPattern;
use super::utils::{current_millis, index_path};

/// The knowledge index store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeIndex {
    /// Learned facts by key
    pub facts: HashMap<String, LearnedFact>,
    /// Learned patterns by ID
    pub patterns: HashMap<String, LearnedPattern>,
    /// Doc snippet cache by topic
    pub doc_cache: HashMap<String, CachedDoc>,
}

impl KnowledgeIndex {
    /// Load from disk
    pub fn load() -> Self {
        let path = index_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(index) => {
                        debug!("Loaded knowledge index");
                        return index;
                    }
                    Err(e) => warn!("Failed to parse knowledge index: {}", e),
                },
                Err(e) => warn!("Failed to read knowledge index: {}", e),
            }
        }
        Self::default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = index_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        debug!(
            "Saved knowledge index ({} facts, {} patterns)",
            self.facts.len(),
            self.patterns.len()
        );
        Ok(())
    }

    /// Add or update a fact
    pub fn learn_fact(&mut self, fact: LearnedFact) {
        let key = fact.key.clone();
        if let Some(existing) = self.facts.get_mut(&key) {
            existing.confirm(&fact.value);
        } else {
            self.facts.insert(key, fact);
        }
    }

    /// Get a fact by key
    pub fn get_fact(&self, key: &str) -> Option<&LearnedFact> {
        self.facts.get(key).filter(|f| !f.is_stale())
    }

    /// Get facts for a domain
    pub fn facts_for_domain(&self, domain: &str) -> Vec<&LearnedFact> {
        self.facts
            .values()
            .filter(|f| f.domain == domain && !f.is_stale())
            .collect()
    }

    /// Add or update a pattern
    pub fn learn_pattern(&mut self, pattern: LearnedPattern) {
        if let Some(existing) = self.patterns.get_mut(&pattern.id) {
            existing.record_success();
        } else {
            self.patterns.insert(pattern.id.clone(), pattern);
        }
    }

    /// Find matching patterns
    pub fn find_patterns(
        &self,
        keywords: &[String],
        domain: &str,
        intent: &str,
    ) -> Vec<&LearnedPattern> {
        self.patterns
            .values()
            .filter(|p| p.matches(keywords, domain, intent))
            .collect()
    }

    /// Find trusted patterns (can answer without LLM)
    pub fn find_trusted_patterns(
        &self,
        keywords: &[String],
        domain: &str,
        intent: &str,
    ) -> Vec<&LearnedPattern> {
        self.find_patterns(keywords, domain, intent)
            .into_iter()
            .filter(|p| p.is_trusted())
            .collect()
    }

    /// Cache a doc snippet
    pub fn cache_doc(&mut self, topic: &str, source: &str, snippet: &str) {
        self.doc_cache.insert(
            topic.to_lowercase(),
            CachedDoc {
                topic: topic.to_string(),
                source: source.to_string(),
                snippet: snippet.to_string(),
                cached_at: current_millis(),
            },
        );
    }

    /// Get cached doc
    pub fn get_cached_doc(&self, topic: &str) -> Option<&CachedDoc> {
        let doc = self.doc_cache.get(&topic.to_lowercase())?;
        // Cache is valid for 24 hours
        if current_millis().saturating_sub(doc.cached_at) < 86400_000 {
            Some(doc)
        } else {
            None
        }
    }

    /// Get stats
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            fact_count: self.facts.len(),
            pattern_count: self.patterns.len(),
            trusted_patterns: self.patterns.values().filter(|p| p.is_trusted()).count(),
            doc_cache_count: self.doc_cache.len(),
        }
    }

    /// Prune stale entries
    pub fn prune_stale(&mut self) {
        // Remove stale facts
        let stale_facts: Vec<String> = self
            .facts
            .iter()
            .filter(|(_, f)| f.is_stale())
            .map(|(k, _)| k.clone())
            .collect();
        for key in stale_facts {
            self.facts.remove(&key);
        }

        // Remove old doc cache (>7 days)
        let old_threshold = current_millis().saturating_sub(604800_000);
        self.doc_cache
            .retain(|_, doc| doc.cached_at > old_threshold);
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub fact_count: usize,
    pub pattern_count: usize,
    pub trusted_patterns: usize,
    pub doc_cache_count: usize,
}

impl std::fmt::Display for IndexStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} facts, {} patterns ({} trusted), {} cached docs",
            self.fact_count, self.pattern_count, self.trusted_patterns, self.doc_cache_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_index() {
        let mut index = KnowledgeIndex::default();

        index.learn_fact(LearnedFact::new("test_key", "test_value", "system"));
        assert!(index.get_fact("test_key").is_some());

        let stats = index.stats();
        assert_eq!(stats.fact_count, 1);
    }
}
