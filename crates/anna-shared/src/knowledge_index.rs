//! Knowledge Index - Anna's compiled knowledge store (v0.0.410).
//!
//! Stores and retrieves learned knowledge:
//! - Facts: Simple key-value knowledge (e.g., "swap_enabled: true")
//! - Patterns: Query patterns with proven solutions
//! - Snippets: Cached doc snippets that worked well
//!
//! Knowledge is accumulated from successful ticket resolutions
//! and allows Anna to answer without re-hitting the LLM.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// A learned fact (simple key-value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedFact {
    /// Fact key (e.g., "swap_enabled", "gpu_vendor")
    pub key: String,
    /// Fact value
    pub value: String,
    /// Domain this fact belongs to
    pub domain: String,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Last verified timestamp
    pub last_verified: u64,
    /// How many times this fact was confirmed
    pub confirmations: u32,
}

impl LearnedFact {
    pub fn new(key: &str, value: &str, domain: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            domain: domain.to_string(),
            confidence: 70,
            last_verified: current_millis(),
            confirmations: 1,
        }
    }

    /// Boost confidence on reconfirmation
    pub fn confirm(&mut self, new_value: &str) {
        if self.value == new_value {
            self.confirmations += 1;
            self.confidence = (self.confidence + 10).min(100);
        } else {
            // Value changed - lower confidence
            self.value = new_value.to_string();
            self.confidence = 60;
        }
        self.last_verified = current_millis();
    }

    /// Check if fact is stale (older than 1 hour)
    pub fn is_stale(&self) -> bool {
        current_millis().saturating_sub(self.last_verified) > 3600_000
    }
}

/// A learned query pattern with solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Pattern ID (hash of keywords)
    pub id: String,
    /// Keywords that trigger this pattern
    pub keywords: Vec<String>,
    /// Domain
    pub domain: String,
    /// Intent
    pub intent: String,
    /// Answer template (with {placeholders})
    pub answer_template: String,
    /// Required probes to run
    pub required_probes: Vec<String>,
    /// Evidence extraction hints
    pub evidence_hints: Vec<EvidenceHint>,
    /// Times this pattern was used successfully
    pub usage_count: u32,
    /// Last success timestamp
    pub last_success: u64,
}

/// Hint for extracting evidence from probe output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceHint {
    /// Probe ID
    pub probe_id: String,
    /// Pattern to match in output
    pub match_pattern: String,
    /// Placeholder name in answer template
    pub placeholder: String,
}

impl LearnedPattern {
    pub fn new(keywords: Vec<String>, domain: &str, intent: &str) -> Self {
        let id = compute_pattern_id(&keywords, domain, intent);
        Self {
            id,
            keywords,
            domain: domain.to_string(),
            intent: intent.to_string(),
            answer_template: String::new(),
            required_probes: vec![],
            evidence_hints: vec![],
            usage_count: 0,
            last_success: 0,
        }
    }

    /// Check if pattern matches query
    pub fn matches(&self, keywords: &[String], domain: &str, intent: &str) -> bool {
        if self.domain != domain || self.intent != intent {
            return false;
        }
        // At least half keywords must match
        let matches = keywords.iter()
            .filter(|k| self.keywords.iter().any(|pk| pk.to_lowercase() == k.to_lowercase()))
            .count();
        matches >= 1 && matches >= keywords.len() / 2
    }

    /// Record successful usage
    pub fn record_success(&mut self) {
        self.usage_count += 1;
        self.last_success = current_millis();
    }

    /// Check if pattern is trusted enough for direct answers
    pub fn is_trusted(&self) -> bool {
        self.usage_count >= 3
    }
}

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

/// Cached documentation snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDoc {
    pub topic: String,
    pub source: String,
    pub snippet: String,
    pub cached_at: u64,
}

impl KnowledgeIndex {
    /// Load from disk
    pub fn load() -> Self {
        let path = index_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(json) => {
                    match serde_json::from_str(&json) {
                        Ok(index) => {
                            debug!("Loaded knowledge index");
                            return index;
                        }
                        Err(e) => warn!("Failed to parse knowledge index: {}", e),
                    }
                }
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
        debug!("Saved knowledge index ({} facts, {} patterns)", self.facts.len(), self.patterns.len());
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
        self.facts.values()
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
    pub fn find_patterns(&self, keywords: &[String], domain: &str, intent: &str) -> Vec<&LearnedPattern> {
        self.patterns.values()
            .filter(|p| p.matches(keywords, domain, intent))
            .collect()
    }

    /// Find trusted patterns (can answer without LLM)
    pub fn find_trusted_patterns(&self, keywords: &[String], domain: &str, intent: &str) -> Vec<&LearnedPattern> {
        self.find_patterns(keywords, domain, intent)
            .into_iter()
            .filter(|p| p.is_trusted())
            .collect()
    }

    /// Cache a doc snippet
    pub fn cache_doc(&mut self, topic: &str, source: &str, snippet: &str) {
        self.doc_cache.insert(topic.to_lowercase(), CachedDoc {
            topic: topic.to_string(),
            source: source.to_string(),
            snippet: snippet.to_string(),
            cached_at: current_millis(),
        });
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
        let stale_facts: Vec<String> = self.facts.iter()
            .filter(|(_, f)| f.is_stale())
            .map(|(k, _)| k.clone())
            .collect();
        for key in stale_facts {
            self.facts.remove(&key);
        }

        // Remove old doc cache (>7 days)
        let old_threshold = current_millis().saturating_sub(604800_000);
        self.doc_cache.retain(|_, doc| doc.cached_at > old_threshold);
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

/// Compute pattern ID from keywords
fn compute_pattern_id(keywords: &[String], domain: &str, intent: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    domain.hash(&mut hasher);
    intent.hash(&mut hasher);
    for kw in keywords {
        kw.to_lowercase().hash(&mut hasher);
    }
    format!("pat_{:016x}", hasher.finish())
}

/// Get index file path
fn index_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".anna")
        .join("knowledge_index.json")
}

fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learned_fact() {
        let mut fact = LearnedFact::new("swap_enabled", "true", "system");
        assert_eq!(fact.confirmations, 1);

        fact.confirm("true");
        assert_eq!(fact.confirmations, 2);
        assert!(fact.confidence > 70);
    }

    #[test]
    fn test_learned_pattern() {
        let pattern = LearnedPattern::new(
            vec!["disk".to_string(), "space".to_string()],
            "storage",
            "diagnose",
        );

        assert!(pattern.matches(&["disk".to_string()], "storage", "diagnose"));
        assert!(!pattern.matches(&["disk".to_string()], "network", "diagnose"));
    }

    #[test]
    fn test_pattern_trust() {
        let mut pattern = LearnedPattern::new(
            vec!["test".to_string()],
            "system",
            "diagnose",
        );

        assert!(!pattern.is_trusted());

        pattern.record_success();
        pattern.record_success();
        pattern.record_success();

        assert!(pattern.is_trusted());
    }

    #[test]
    fn test_knowledge_index() {
        let mut index = KnowledgeIndex::default();

        index.learn_fact(LearnedFact::new("test_key", "test_value", "system"));
        assert!(index.get_fact("test_key").is_some());

        let stats = index.stats();
        assert_eq!(stats.fact_count, 1);
    }
}
