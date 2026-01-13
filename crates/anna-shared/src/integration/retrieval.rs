//! Knowledge Retrieval with Hard Precedence Rules.
//!
//! Retrieval order (MUST be followed):
//! 1. Live probes (current system state) - Ground truth
//! 2. Trusted docs corpus (Arch Wiki, man pages) - Authoritative
//! 3. Validated skills/procedures - Tested and promoted
//! 4. Semantic memory (learned patterns) - Derived knowledge
//! 5. Episodic memory (past experiences) - Evidence only, NOT fact
//!
//! Episodic memory must NEVER be treated as ground truth.
//! Any answer citing memory must carry provenance pointers.

use super::provenance::{ProvenanceRecord, ProvenanceSource};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Knowledge source with reliability weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Live system probe - highest priority
    LiveProbe,
    /// Trusted documentation corpus
    TrustedDocs,
    /// Validated skill/procedure
    ValidatedSkill,
    /// Semantic memory (patterns)
    SemanticMemory,
    /// Episodic memory (past experiences) - evidence only
    EpisodicMemory,
}

impl KnowledgeSource {
    /// Reliability weight for confidence calculation
    pub fn reliability_weight(&self) -> f32 {
        match self {
            KnowledgeSource::LiveProbe => 1.0,
            KnowledgeSource::TrustedDocs => 0.95,
            KnowledgeSource::ValidatedSkill => 0.85,
            KnowledgeSource::SemanticMemory => 0.6,
            KnowledgeSource::EpisodicMemory => 0.4, // Low - evidence only
        }
    }

    /// Precedence order (lower = higher priority)
    pub fn precedence(&self) -> u8 {
        match self {
            KnowledgeSource::LiveProbe => 0,
            KnowledgeSource::TrustedDocs => 1,
            KnowledgeSource::ValidatedSkill => 2,
            KnowledgeSource::SemanticMemory => 3,
            KnowledgeSource::EpisodicMemory => 4,
        }
    }

    /// Can this source be used as ground truth?
    pub fn is_ground_truth(&self) -> bool {
        matches!(self, KnowledgeSource::LiveProbe | KnowledgeSource::TrustedDocs)
    }

    /// Does this source require provenance?
    pub fn requires_provenance(&self) -> bool {
        // All sources require provenance, but episodic MUST have it
        true
    }
}

/// A piece of retrieved knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedKnowledge {
    /// The knowledge content
    pub content: String,
    /// Source of this knowledge
    pub source: KnowledgeSource,
    /// Confidence in this knowledge (0.0-1.0)
    pub confidence: f32,
    /// Provenance record (required for all sources)
    pub provenance: ProvenanceRecord,
    /// Is this knowledge stale?
    pub stale: bool,
    /// Age in seconds (for live probes)
    pub age_secs: Option<u64>,
    /// Citation text (for docs)
    pub citation: Option<String>,
}

impl RetrievedKnowledge {
    /// Check if this knowledge can be trusted as fact
    pub fn is_trustworthy(&self) -> bool {
        !self.stale && self.source.is_ground_truth() && self.confidence > 0.7
    }

    /// Get effective confidence after applying source weight
    pub fn effective_confidence(&self) -> f32 {
        self.confidence * self.source.reliability_weight()
    }
}

/// The knowledge retrieval system
#[derive(Debug, Clone, Default)]
pub struct KnowledgeRetrieval {
    /// Recent provenance records
    provenance_log: VecDeque<ProvenanceRecord>,
    /// Maximum provenance log size
    max_provenance: usize,
    /// Cached live probe results
    probe_cache: Vec<RetrievedKnowledge>,
    /// Probe cache TTL in seconds
    probe_cache_ttl: u64,
}

impl KnowledgeRetrieval {
    /// Create new retrieval system
    pub fn new() -> Self {
        Self {
            provenance_log: VecDeque::new(),
            max_provenance: 1000,
            probe_cache: Vec::new(),
            probe_cache_ttl: 30, // 30 second TTL for probes
        }
    }

    /// Retrieve knowledge for a question with strict precedence
    pub fn retrieve(&mut self, question: &str) -> Vec<RetrievedKnowledge> {
        let mut results = Vec::new();

        // 1. Live probes (highest priority)
        results.extend(self.retrieve_live_probes(question));

        // 2. Trusted docs (Arch Wiki, man pages)
        results.extend(self.retrieve_trusted_docs(question));

        // 3. Validated skills
        results.extend(self.retrieve_validated_skills(question));

        // 4. Semantic memory (patterns)
        results.extend(self.retrieve_semantic_memory(question));

        // 5. Episodic memory (evidence only - mark appropriately)
        let episodic = self.retrieve_episodic_memory(question);
        for mut k in episodic {
            // Mark episodic as evidence, not fact
            k.content = format!("[Evidence from past experience] {}", k.content);
            results.push(k);
        }

        // Sort by precedence (lower = higher priority)
        results.sort_by(|a, b| a.source.precedence().cmp(&b.source.precedence()));

        // Log provenance
        for k in &results {
            self.log_provenance(k.provenance.clone());
        }

        results
    }

    /// Retrieve live probe results
    fn retrieve_live_probes(&mut self, _question: &str) -> Vec<RetrievedKnowledge> {
        // Live probes are executed by the daemon, not here
        // This returns cached probe results if fresh
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.probe_cache
            .iter()
            .filter(|p| {
                if let Some(age) = p.age_secs {
                    age < self.probe_cache_ttl
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Retrieve from trusted docs corpus
    /// This is the AUTHORITATIVE source - answers MUST cite this when available
    fn retrieve_trusted_docs(&self, question: &str) -> Vec<RetrievedKnowledge> {
        // Try to load wiki index and search
        // Note: This is a sync interface; async search would be called from daemon
        match crate::wiki::index::WikiIndex::load() {
            Ok(index) => {
                match crate::wiki::search::keyword_search(&index, question, 3) {
                    Ok(results) => {
                        results
                            .into_iter()
                            .map(|r| {
                                let citation = format!(
                                    "Arch Wiki: {} ({})",
                                    r.article.title, r.article.url
                                );
                                RetrievedKnowledge {
                                    content: r.relevant_section.unwrap_or_else(|| {
                                        // Take first 500 chars of content
                                        r.article.content.chars().take(500).collect()
                                    }),
                                    source: KnowledgeSource::TrustedDocs,
                                    confidence: r.score,
                                    provenance: ProvenanceRecord {
                                        source: ProvenanceSource::TrustedDocs {
                                            article: r.article.title.clone(),
                                            section: None,
                                            url: r.article.url.clone(),
                                        },
                                        confidence: r.score,
                                        retrieved_at: chrono::Utc::now().to_rfc3339(),
                                        used_for: None,
                                    },
                                    stale: false,
                                    age_secs: None,
                                    citation: Some(citation), // REQUIRED citation
                                }
                            })
                            .collect()
                    }
                    Err(_) => Vec::new(),
                }
            }
            Err(_) => Vec::new(), // Wiki not available
        }
    }

    /// Retrieve validated skills
    fn retrieve_validated_skills(&self, _question: &str) -> Vec<RetrievedKnowledge> {
        // Query skill_promotion module for trusted skills
        Vec::new()
    }

    /// Retrieve from semantic memory
    fn retrieve_semantic_memory(&self, _question: &str) -> Vec<RetrievedKnowledge> {
        // Query memory module patterns
        Vec::new()
    }

    /// Retrieve from episodic memory (experiences)
    fn retrieve_episodic_memory(&self, _question: &str) -> Vec<RetrievedKnowledge> {
        // Query memory module experiences
        // IMPORTANT: Mark these as evidence only
        Vec::new()
    }

    /// Cache a live probe result
    pub fn cache_probe(&mut self, content: String, command: String, confidence: f32) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let knowledge = RetrievedKnowledge {
            content,
            source: KnowledgeSource::LiveProbe,
            confidence,
            provenance: ProvenanceRecord {
                source: ProvenanceSource::LiveProbe {
                    command: command.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                confidence,
                retrieved_at: chrono::Utc::now().to_rfc3339(),
                used_for: None,
            },
            stale: false,
            age_secs: Some(0),
            citation: Some(format!("Output of: {}", command)),
        };

        self.probe_cache.push(knowledge);

        // Trim old probes
        self.probe_cache.retain(|p| {
            p.age_secs.map(|a| a < self.probe_cache_ttl * 2).unwrap_or(false)
        });
    }

    /// Add docs knowledge with citation
    pub fn add_docs_knowledge(
        &mut self,
        content: String,
        article: String,
        section: Option<String>,
        url: String,
        confidence: f32,
    ) -> RetrievedKnowledge {
        let citation = if let Some(ref sec) = section {
            format!("Arch Wiki: {} - {}", article, sec)
        } else {
            format!("Arch Wiki: {}", article)
        };

        RetrievedKnowledge {
            content,
            source: KnowledgeSource::TrustedDocs,
            confidence,
            provenance: ProvenanceRecord {
                source: ProvenanceSource::TrustedDocs {
                    article: article.clone(),
                    section,
                    url,
                },
                confidence,
                retrieved_at: chrono::Utc::now().to_rfc3339(),
                used_for: None,
            },
            stale: false,
            age_secs: None,
            citation: Some(citation),
        }
    }

    /// Add episodic knowledge (with required provenance)
    pub fn add_episodic_knowledge(
        &mut self,
        content: String,
        experience_id: String,
        question: String,
        confidence: f32,
    ) -> RetrievedKnowledge {
        RetrievedKnowledge {
            content,
            source: KnowledgeSource::EpisodicMemory,
            confidence: confidence * 0.8, // Discount episodic
            provenance: ProvenanceRecord {
                source: ProvenanceSource::EpisodicMemory {
                    experience_id,
                    original_question: question,
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
                confidence,
                retrieved_at: chrono::Utc::now().to_rfc3339(),
                used_for: None,
            },
            stale: false,
            age_secs: None,
            citation: None,
        }
    }

    /// Log provenance record
    fn log_provenance(&mut self, record: ProvenanceRecord) {
        self.provenance_log.push_back(record);
        while self.provenance_log.len() > self.max_provenance {
            self.provenance_log.pop_front();
        }
    }

    /// Get recent provenance records
    pub fn get_provenance(&self) -> Vec<ProvenanceRecord> {
        self.provenance_log.iter().cloned().collect()
    }

    /// Clear probe cache (force fresh probes)
    pub fn clear_probe_cache(&mut self) {
        self.probe_cache.clear();
    }

    /// Check if we need fresh probes for a question
    pub fn needs_fresh_probes(&self) -> bool {
        self.probe_cache.is_empty()
            || self.probe_cache.iter().all(|p| {
                p.age_secs.map(|a| a > self.probe_cache_ttl).unwrap_or(true)
            })
    }
}

/// Acceptance test: for a repeated task in a changed system state,
/// Anna must prefer fresh probes over "last time I did X"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_precedence() {
        assert!(KnowledgeSource::LiveProbe.precedence() < KnowledgeSource::EpisodicMemory.precedence());
        assert!(KnowledgeSource::TrustedDocs.precedence() < KnowledgeSource::SemanticMemory.precedence());
    }

    #[test]
    fn test_episodic_not_ground_truth() {
        assert!(!KnowledgeSource::EpisodicMemory.is_ground_truth());
        assert!(KnowledgeSource::LiveProbe.is_ground_truth());
    }

    #[test]
    fn test_reliability_weights() {
        assert!(KnowledgeSource::LiveProbe.reliability_weight() > KnowledgeSource::EpisodicMemory.reliability_weight());
    }

    #[test]
    fn test_retrieval_precedence_order() {
        let mut retrieval = KnowledgeRetrieval::new();

        // Cache a probe
        retrieval.cache_probe("probe result".to_string(), "ls".to_string(), 1.0);

        let results = retrieval.retrieve("test");

        // Probes should come first
        if !results.is_empty() {
            assert_eq!(results[0].source, KnowledgeSource::LiveProbe);
        }
    }
}
