//! Ambiguity Resolver - v0.25.0
//!
//! Handles ambiguous queries by identifying candidates and prompting user.
//! No guessing - when unclear, asks for clarification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::protocol_v25::{
    AmbiguityCandidate, AmbiguityResolution, AmbiguityType, DetectedAmbiguity, RankedEntity,
    RankedEntityType, RelevanceScore,
};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Minimum confidence to consider a match unambiguous
pub const DEFAULT_UNAMBIGUOUS_THRESHOLD: f32 = 0.85;

/// Maximum candidates to present to user
pub const DEFAULT_MAX_CANDIDATES: usize = 5;

/// Maximum remembered resolutions
pub const DEFAULT_MAX_REMEMBERED: usize = 100;

/// Configuration for ambiguity resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityConfig {
    /// Confidence threshold for unambiguous match
    pub unambiguous_threshold: f32,
    /// Max candidates to show
    pub max_candidates: usize,
    /// Max remembered resolutions
    pub max_remembered: usize,
    /// Ask for confirmation even with high confidence
    pub always_confirm: bool,
}

impl Default for AmbiguityConfig {
    fn default() -> Self {
        Self {
            unambiguous_threshold: DEFAULT_UNAMBIGUOUS_THRESHOLD,
            max_candidates: DEFAULT_MAX_CANDIDATES,
            max_remembered: DEFAULT_MAX_REMEMBERED,
            always_confirm: false,
        }
    }
}

// ============================================================================
// AMBIGUITY DETECTOR
// ============================================================================

/// Detects ambiguities in queries
#[derive(Debug, Clone, Default)]
pub struct AmbiguityDetector {
    /// Configuration
    pub config: AmbiguityConfig,
}

impl AmbiguityDetector {
    /// Create with config
    pub fn with_config(config: AmbiguityConfig) -> Self {
        Self { config }
    }

    /// Check if a term is ambiguous given candidates
    pub fn detect(&self, term: &str, candidates: Vec<RankedEntity>, query_context: &str) -> Option<DetectedAmbiguity> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            // Single candidate - check if high enough confidence
            let confidence = candidates[0].relevance.score;
            if confidence >= self.config.unambiguous_threshold && !self.config.always_confirm {
                return None;
            }
        }

        // Multiple candidates or low confidence - ambiguous
        let ambiguity_type = self.classify_ambiguity(&candidates);
        let amb_candidates = self.rank_candidates(candidates);

        // If top candidate is clearly ahead, might not be ambiguous
        if amb_candidates.len() >= 2 {
            let gap = amb_candidates[0].confidence - amb_candidates[1].confidence;
            if gap > 0.5 && !self.config.always_confirm {
                return None;
            }
        }

        Some(DetectedAmbiguity {
            id: uuid::Uuid::new_v4().to_string(),
            ambiguity_type,
            term: term.to_string(),
            candidates: amb_candidates,
            confidence: 0.9, // High confidence that it IS ambiguous
            query_context: query_context.to_string(),
        })
    }

    /// Classify the type of ambiguity
    fn classify_ambiguity(&self, candidates: &[RankedEntity]) -> AmbiguityType {
        if candidates.is_empty() {
            return AmbiguityType::PolysemousTerm;
        }

        // Check if all same type
        let first_type = &candidates[0].entity_type;
        let all_same = candidates.iter().all(|c| &c.entity_type == first_type);

        if all_same {
            match first_type {
                RankedEntityType::Application => AmbiguityType::MultipleApps,
                RankedEntityType::Package => AmbiguityType::MultiplePackages,
                RankedEntityType::Service => AmbiguityType::MultipleServices,
                _ => AmbiguityType::PolysemousTerm,
            }
        } else {
            AmbiguityType::PolysemousTerm
        }
    }

    /// Convert and rank candidates
    fn rank_candidates(&self, mut entities: Vec<RankedEntity>) -> Vec<AmbiguityCandidate> {
        // Sort by relevance
        entities.sort_by(|a, b| {
            b.relevance.score.partial_cmp(&a.relevance.score).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top N
        entities.truncate(self.config.max_candidates);

        entities
            .into_iter()
            .map(|e| {
                let confidence = e.relevance.score;
                let reasoning = self.generate_reasoning(&e);
                AmbiguityCandidate {
                    entity: e,
                    confidence,
                    reasoning,
                }
            })
            .collect()
    }

    /// Generate reasoning for why this candidate matches
    fn generate_reasoning(&self, entity: &RankedEntity) -> String {
        let mut parts = Vec::new();

        if entity.usage_count > 0 {
            parts.push(format!("used {} times", entity.usage_count));
        }

        if let Some(last) = entity.last_used {
            let now = chrono::Utc::now().timestamp();
            let age_mins = (now - last) / 60;
            if age_mins < 60 {
                parts.push(format!("used {} mins ago", age_mins));
            } else if age_mins < 1440 {
                parts.push(format!("used {} hours ago", age_mins / 60));
            }
        }

        if entity.relevance.factors.session_context > 0.5 {
            parts.push("used in current session".to_string());
        }

        if parts.is_empty() {
            "discovered via system probes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

// ============================================================================
// RESOLUTION MEMORY
// ============================================================================

/// Remembers how user resolved ambiguities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolutionMemory {
    /// Term -> resolution mapping
    pub resolutions: HashMap<String, RememberedResolution>,
    /// Configuration
    pub max_entries: usize,
}

/// A remembered resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RememberedResolution {
    /// The term that was ambiguous
    pub term: String,
    /// The entity ID user selected
    pub selected_entity: String,
    /// Context in which this was resolved
    pub context_keywords: Vec<String>,
    /// How many times this resolution was used
    pub use_count: u32,
    /// When this was last used
    pub last_used: i64,
    /// Original ambiguity type
    pub ambiguity_type: AmbiguityType,
}

impl ResolutionMemory {
    /// Create with capacity
    pub fn new(max_entries: usize) -> Self {
        Self {
            resolutions: HashMap::new(),
            max_entries,
        }
    }

    /// Remember a resolution
    pub fn remember(&mut self, _resolution: &AmbiguityResolution, ambiguity: &DetectedAmbiguity, selected: &RankedEntity) {
        let key = self.make_key(&ambiguity.term, &ambiguity.query_context);

        let remembered = RememberedResolution {
            term: ambiguity.term.clone(),
            selected_entity: selected.id.clone(),
            context_keywords: self.extract_keywords(&ambiguity.query_context),
            use_count: 1,
            last_used: chrono::Utc::now().timestamp(),
            ambiguity_type: ambiguity.ambiguity_type.clone(),
        };

        self.resolutions.insert(key, remembered);
        self.trim();
    }

    /// Look up a remembered resolution
    pub fn lookup(&self, term: &str, context: &str) -> Option<&RememberedResolution> {
        // Try exact match first
        let key = self.make_key(term, context);
        if let Some(r) = self.resolutions.get(&key) {
            return Some(r);
        }

        // Try term-only match
        self.resolutions.get(term)
    }

    /// Make a lookup key
    fn make_key(&self, term: &str, context: &str) -> String {
        let keywords = self.extract_keywords(context);
        if keywords.is_empty() {
            term.to_lowercase()
        } else {
            format!("{}:{}", term.to_lowercase(), keywords.join(","))
        }
    }

    /// Extract context keywords
    fn extract_keywords(&self, context: &str) -> Vec<String> {
        context
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .take(3)
            .map(|s| s.to_string())
            .collect()
    }

    /// Trim to max size
    fn trim(&mut self) {
        if self.resolutions.len() <= self.max_entries {
            return;
        }

        // Remove oldest entries
        let mut entries: Vec<_> = self.resolutions.iter().collect();
        entries.sort_by(|a, b| a.1.last_used.cmp(&b.1.last_used));

        let to_remove: Vec<_> = entries
            .iter()
            .take(self.resolutions.len() - self.max_entries)
            .map(|(k, _)| (*k).clone())
            .collect();

        for key in to_remove {
            self.resolutions.remove(&key);
        }
    }

    /// Update use count for a resolution
    pub fn record_use(&mut self, term: &str, context: &str) {
        let key = self.make_key(term, context);
        if let Some(r) = self.resolutions.get_mut(&key) {
            r.use_count += 1;
            r.last_used = chrono::Utc::now().timestamp();
        }
    }
}

// ============================================================================
// PROMPT BUILDER
// ============================================================================

/// Builds user-friendly prompts for disambiguation
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder;

impl PromptBuilder {
    /// Build a disambiguation prompt
    pub fn build_prompt(&self, ambiguity: &DetectedAmbiguity) -> DisambiguationPrompt {
        let question = self.format_question(ambiguity);
        let options: Vec<_> = ambiguity
            .candidates
            .iter()
            .enumerate()
            .map(|(i, c)| PromptOption {
                index: i,
                label: c.entity.display_name.clone(),
                description: c.reasoning.clone(),
                entity_id: c.entity.id.clone(),
            })
            .collect();

        DisambiguationPrompt {
            ambiguity_id: ambiguity.id.clone(),
            question,
            options,
            allow_remember: true,
        }
    }

    /// Format the question based on ambiguity type
    fn format_question(&self, ambiguity: &DetectedAmbiguity) -> String {
        match ambiguity.ambiguity_type {
            AmbiguityType::MultipleApps => {
                format!("Which '{}' did you mean?", ambiguity.term)
            }
            AmbiguityType::MultiplePackages => {
                format!("Multiple packages match '{}'. Which one?", ambiguity.term)
            }
            AmbiguityType::MultipleServices => {
                format!("Multiple services match '{}'. Which one?", ambiguity.term)
            }
            AmbiguityType::PolysemousTerm => {
                format!("'{}' could refer to several things. Which one?", ambiguity.term)
            }
            AmbiguityType::UnclearScope => {
                format!("Should I look for '{}' system-wide or in your user config?", ambiguity.term)
            }
            AmbiguityType::UnspecifiedVariant => {
                format!("Multiple variants of '{}' exist. Which one?", ambiguity.term)
            }
        }
    }
}

/// A disambiguation prompt for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationPrompt {
    /// Ambiguity ID (for resolution)
    pub ambiguity_id: String,
    /// The question to ask
    pub question: String,
    /// Available options
    pub options: Vec<PromptOption>,
    /// Whether to offer "remember this choice"
    pub allow_remember: bool,
}

/// A single option in a disambiguation prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOption {
    /// Option index
    pub index: usize,
    /// Display label
    pub label: String,
    /// Description/reasoning
    pub description: String,
    /// Entity ID this represents
    pub entity_id: String,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::protocol_v25::{DiscoverySource, RelevanceFactors};

    fn make_entity(id: &str, name: &str, score: f32) -> RankedEntity {
        RankedEntity {
            id: id.to_string(),
            entity_type: RankedEntityType::Application,
            display_name: name.to_string(),
            relevance: RelevanceScore::new(score, RelevanceFactors::default()),
            usage_count: 10,
            last_used: Some(chrono::Utc::now().timestamp()),
            discovered_via: DiscoverySource::Probe { probe_id: "test".to_string() },
        }
    }

    #[test]
    fn test_no_ambiguity_single_high_confidence() {
        let detector = AmbiguityDetector::default();
        let candidates = vec![make_entity("firefox", "Firefox", 0.95)];

        let result = detector.detect("firefox", candidates, "open firefox");
        assert!(result.is_none()); // Clear match, no ambiguity
    }

    #[test]
    fn test_ambiguity_multiple_candidates() {
        let detector = AmbiguityDetector::default();
        let candidates = vec![
            make_entity("code-oss", "Code - OSS", 0.7),
            make_entity("code", "Visual Studio Code", 0.65),
        ];

        let result = detector.detect("code", candidates, "open code");
        assert!(result.is_some());

        let amb = result.unwrap();
        assert_eq!(amb.ambiguity_type, AmbiguityType::MultipleApps);
        assert_eq!(amb.candidates.len(), 2);
    }

    #[test]
    fn test_resolution_memory() {
        let mut memory = ResolutionMemory::new(10);

        let ambiguity = DetectedAmbiguity {
            id: "amb-001".to_string(),
            ambiguity_type: AmbiguityType::MultipleApps,
            term: "code".to_string(),
            candidates: vec![],
            confidence: 0.9,
            query_context: "open code editor".to_string(),
        };

        let resolution = AmbiguityResolution {
            ambiguity_id: "amb-001".to_string(),
            selected_index: 0,
            remember: true,
            resolved_at: chrono::Utc::now().timestamp(),
        };

        let entity = make_entity("code-oss", "Code - OSS", 0.7);
        memory.remember(&resolution, &ambiguity, &entity);

        let lookup = memory.lookup("code", "open code editor");
        assert!(lookup.is_some());
        assert_eq!(lookup.unwrap().selected_entity, "code-oss");
    }

    #[test]
    fn test_prompt_builder() {
        let builder = PromptBuilder;

        let ambiguity = DetectedAmbiguity {
            id: "amb-001".to_string(),
            ambiguity_type: AmbiguityType::MultipleApps,
            term: "code".to_string(),
            candidates: vec![
                AmbiguityCandidate {
                    entity: make_entity("code-oss", "Code - OSS", 0.7),
                    confidence: 0.7,
                    reasoning: "used 5 times".to_string(),
                },
            ],
            confidence: 0.9,
            query_context: "open code".to_string(),
        };

        let prompt = builder.build_prompt(&ambiguity);
        assert!(prompt.question.contains("code"));
        assert_eq!(prompt.options.len(), 1);
    }

    #[test]
    fn test_ambiguity_type_classification() {
        let detector = AmbiguityDetector::default();

        let apps = vec![
            make_entity("app1", "App 1", 0.5),
            make_entity("app2", "App 2", 0.5),
        ];
        let amb = detector.detect("test", apps, "test").unwrap();
        assert_eq!(amb.ambiguity_type, AmbiguityType::MultipleApps);
    }
}
