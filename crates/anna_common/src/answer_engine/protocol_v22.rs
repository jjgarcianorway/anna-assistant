//! Protocol v0.22.0 - Fact Brain & Question Decomposition
//!
//! Key principles:
//! - TTL-based fact management with automatic expiry
//! - Question decomposition into answerable subquestions
//! - Validated facts with confidence tracking
//! - Semantic fact linking for related knowledge
//!
//! Architecture:
//! 1. Fact Brain: Central knowledge hub with TTLs
//! 2. Question Analyzer: Decompose complex questions
//! 3. Fact Validator: Verify facts before storage
//! 4. Semantic Linker: Connect related facts

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TTL categories for different fact types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactTtlCategory {
    /// Static facts that rarely change (e.g., CPU model) - 7 days
    Static,
    /// Semi-static facts (e.g., installed packages) - 24 hours
    SemiStatic,
    /// Dynamic facts (e.g., disk usage) - 1 hour
    Dynamic,
    /// Volatile facts (e.g., process list) - 5 minutes
    Volatile,
    /// User-provided facts - 30 days
    UserProvided,
}

impl FactTtlCategory {
    /// Get TTL duration for this category
    pub fn ttl(&self) -> Duration {
        match self {
            Self::Static => Duration::days(7),
            Self::SemiStatic => Duration::hours(24),
            Self::Dynamic => Duration::hours(1),
            Self::Volatile => Duration::minutes(5),
            Self::UserProvided => Duration::days(30),
        }
    }

    /// Get TTL in seconds
    pub fn ttl_seconds(&self) -> i64 {
        self.ttl().num_seconds()
    }

    /// Classify entity/attribute pair
    pub fn classify(entity: &str, attribute: &str) -> Self {
        let prefix = entity.split(':').next().unwrap_or("");
        match (prefix, attribute) {
            // Static facts
            ("cpu", "model") | ("cpu", "vendor") | ("cpu", "cores") => Self::Static,
            ("gpu", "model") | ("gpu", "vendor") | ("gpu", "vram") => Self::Static,
            ("system", "hostname") | ("system", "os") | ("system", "arch") => Self::Static,

            // Semi-static facts
            ("pkg", _) | ("svc", "enabled") | ("cfg", _) => Self::SemiStatic,
            ("disk", "model") | ("disk", "size") => Self::SemiStatic,
            ("net", "interface") | ("net", "mac") => Self::SemiStatic,

            // Dynamic facts
            ("disk", "used") | ("disk", "free") => Self::Dynamic,
            ("fs", "used") | ("fs", "free") => Self::Dynamic,
            ("svc", "status") => Self::Dynamic,
            ("cpu", "usage") | ("cpu", "temp") => Self::Dynamic,

            // Volatile facts
            ("proc", _) | ("net", "connections") => Self::Volatile,
            ("cpu", "load") | ("mem", "used") => Self::Volatile,

            // Default to semi-static
            _ => Self::SemiStatic,
        }
    }
}

/// A fact with TTL metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlFact {
    /// Entity identifier
    pub entity: String,
    /// Attribute name
    pub attribute: String,
    /// Value
    pub value: String,
    /// TTL category
    pub ttl_category: FactTtlCategory,
    /// When the fact was created
    pub created_at: DateTime<Utc>,
    /// When the fact expires
    pub expires_at: DateTime<Utc>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Validation status
    pub validation: ValidationStatus,
    /// Source of the fact
    pub source: FactSourceV22,
}

impl TtlFact {
    /// Create a new fact with automatic TTL
    pub fn new(
        entity: String,
        attribute: String,
        value: String,
        confidence: f32,
        source: FactSourceV22,
    ) -> Self {
        let ttl_category = FactTtlCategory::classify(&entity, &attribute);
        let created_at = Utc::now();
        let expires_at = created_at + ttl_category.ttl();

        Self {
            entity,
            attribute,
            value,
            ttl_category,
            created_at,
            expires_at,
            confidence,
            validation: ValidationStatus::Pending,
            source,
        }
    }

    /// Check if fact is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if fact is close to expiry (within 10% of TTL)
    pub fn is_expiring_soon(&self) -> bool {
        let remaining = self.expires_at.signed_duration_since(Utc::now());
        let ttl = self.ttl_category.ttl();
        remaining < ttl / 10
    }

    /// Refresh the fact with a new TTL
    pub fn refresh(&mut self) {
        self.created_at = Utc::now();
        self.expires_at = self.created_at + self.ttl_category.ttl();
    }

    /// Mark as validated
    pub fn mark_validated(&mut self, result: ValidationResult) {
        self.validation = match result {
            ValidationResult::Valid => ValidationStatus::Valid,
            ValidationResult::Invalid(reason) => ValidationStatus::Invalid(reason),
            ValidationResult::Uncertain => ValidationStatus::Uncertain,
        };
    }
}

/// Source of a fact
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FactSourceV22 {
    /// From a system probe
    #[serde(rename = "probe")]
    Probe { probe_id: String, timestamp: DateTime<Utc> },
    /// From LLM inference
    #[serde(rename = "llm")]
    Llm { model: String, reasoning: String },
    /// User-provided
    #[serde(rename = "user")]
    User { user_id: Option<String> },
    /// Derived from other facts
    #[serde(rename = "derived")]
    Derived { from_facts: Vec<String> },
}

/// Validation status of a fact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// Not yet validated
    Pending,
    /// Validated and correct
    Valid,
    /// Validation failed
    Invalid(String),
    /// Validation inconclusive
    Uncertain,
}

/// Result of fact validation
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Uncertain,
}

/// Question decomposition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionDecomposition {
    /// Original question
    pub original: String,
    /// Decomposed subquestions
    pub subquestions: Vec<Subquestion>,
    /// Required facts to answer
    pub required_facts: Vec<RequiredFact>,
    /// Synthesis strategy
    pub synthesis: SynthesisStrategy,
}

/// A subquestion from decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subquestion {
    /// Subquestion ID
    pub id: String,
    /// The subquestion text
    pub question: String,
    /// Type of subquestion
    pub question_type: QuestionType,
    /// Dependencies on other subquestions
    pub depends_on: Vec<String>,
    /// Facts needed to answer this
    pub required_facts: Vec<String>,
    /// Priority (1 = highest)
    pub priority: u8,
}

/// Type of question
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    /// Simple factual lookup
    FactLookup,
    /// Comparison between values
    Comparison,
    /// Calculation or aggregation
    Calculation,
    /// Explanation or reasoning
    Explanation,
    /// Status check
    StatusCheck,
    /// Configuration query
    ConfigQuery,
}

impl QuestionType {
    /// Can this question type be answered from facts alone?
    pub fn answerable_from_facts(&self) -> bool {
        matches!(self, Self::FactLookup | Self::Comparison | Self::StatusCheck)
    }
}

/// A required fact for answering a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFact {
    /// Entity pattern (may include wildcards)
    pub entity_pattern: String,
    /// Attribute name
    pub attribute: String,
    /// Whether this fact is required or optional
    pub required: bool,
    /// Probe that can provide this fact
    pub probe_id: Option<String>,
}

/// Strategy for synthesizing the final answer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "strategy")]
pub enum SynthesisStrategy {
    /// Direct answer from a single fact
    #[serde(rename = "direct")]
    Direct { fact_key: String },
    /// Aggregate multiple facts
    #[serde(rename = "aggregate")]
    Aggregate { operation: AggregateOp, fact_keys: Vec<String> },
    /// Compare values
    #[serde(rename = "compare")]
    Compare { left: String, right: String },
    /// Template-based answer
    #[serde(rename = "template")]
    Template { template: String, bindings: HashMap<String, String> },
    /// LLM synthesis required
    #[serde(rename = "llm_synthesis")]
    LlmSynthesis { prompt_hint: String },
}

/// Aggregation operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateOp {
    Sum,
    Average,
    Count,
    Min,
    Max,
    List,
}

/// Fact Brain state
#[derive(Debug, Clone)]
pub struct FactBrain {
    /// Facts indexed by entity:attribute
    pub facts: HashMap<String, TtlFact>,
    /// Semantic links between facts
    pub links: Vec<SemanticLink>,
    /// Pending validations
    pub pending_validations: Vec<String>,
}

impl FactBrain {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            links: Vec::new(),
            pending_validations: Vec::new(),
        }
    }

    /// Get fact key
    fn fact_key(entity: &str, attribute: &str) -> String {
        format!("{}:{}", entity, attribute)
    }

    /// Store a fact
    pub fn store(&mut self, fact: TtlFact) {
        let key = Self::fact_key(&fact.entity, &fact.attribute);
        if fact.validation == ValidationStatus::Pending {
            self.pending_validations.push(key.clone());
        }
        self.facts.insert(key, fact);
    }

    /// Get a fact
    pub fn get(&self, entity: &str, attribute: &str) -> Option<&TtlFact> {
        let key = Self::fact_key(entity, attribute);
        self.facts.get(&key).filter(|f| !f.is_expired())
    }

    /// Get all facts matching entity prefix
    pub fn get_by_prefix(&self, prefix: &str) -> Vec<&TtlFact> {
        self.facts
            .values()
            .filter(|f| f.entity.starts_with(prefix) && !f.is_expired())
            .collect()
    }

    /// Remove expired facts
    pub fn cleanup_expired(&mut self) -> usize {
        let before = self.facts.len();
        self.facts.retain(|_, f| !f.is_expired());
        before - self.facts.len()
    }

    /// Get facts expiring soon
    pub fn get_expiring_soon(&self) -> Vec<&TtlFact> {
        self.facts
            .values()
            .filter(|f| f.is_expiring_soon() && !f.is_expired())
            .collect()
    }

    /// Count facts by TTL category
    pub fn count_by_category(&self) -> HashMap<FactTtlCategory, usize> {
        let mut counts = HashMap::new();
        for fact in self.facts.values() {
            *counts.entry(fact.ttl_category).or_insert(0) += 1;
        }
        counts
    }

    /// Add semantic link
    pub fn add_link(&mut self, link: SemanticLink) {
        self.links.push(link);
    }

    /// Get related facts
    pub fn get_related(&self, entity: &str, attribute: &str) -> Vec<&TtlFact> {
        let key = Self::fact_key(entity, attribute);
        let mut related = Vec::new();

        for link in &self.links {
            if link.from_fact == key {
                if let Some(fact) = self.facts.get(&link.to_fact) {
                    if !fact.is_expired() {
                        related.push(fact);
                    }
                }
            } else if link.to_fact == key {
                if let Some(fact) = self.facts.get(&link.from_fact) {
                    if !fact.is_expired() {
                        related.push(fact);
                    }
                }
            }
        }

        related
    }
}

impl Default for FactBrain {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic link between facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticLink {
    /// Source fact key (entity:attribute)
    pub from_fact: String,
    /// Target fact key
    pub to_fact: String,
    /// Type of relationship
    pub relation: SemanticRelation,
    /// Link strength (0.0-1.0)
    pub strength: f32,
}

/// Type of semantic relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRelation {
    /// Facts are about the same entity
    SameEntity,
    /// One fact depends on another
    DependsOn,
    /// Facts are related by topic
    RelatedTopic,
    /// One fact implies another
    Implies,
    /// Facts conflict with each other
    Conflicts,
}

/// Question analyzer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeQuestionRequest {
    /// The question to analyze
    pub question: String,
    /// Available facts in the brain
    pub available_facts: Vec<String>,
    /// Available probes
    pub available_probes: Vec<String>,
}

/// Question analyzer response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeQuestionResponse {
    /// Decomposition result
    pub decomposition: QuestionDecomposition,
    /// Confidence in decomposition (0.0-1.0)
    pub confidence: f32,
    /// Alternative interpretations
    pub alternatives: Vec<String>,
}

/// Constants for v0.22.0
pub const MAX_SUBQUESTIONS: usize = 5;
pub const DEFAULT_FACT_CONFIDENCE: f32 = 0.8;
pub const MIN_LINK_STRENGTH: f32 = 0.5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_category_classification() {
        assert_eq!(
            FactTtlCategory::classify("cpu:0", "model"),
            FactTtlCategory::Static
        );
        assert_eq!(
            FactTtlCategory::classify("pkg:vim", "version"),
            FactTtlCategory::SemiStatic
        );
        assert_eq!(
            FactTtlCategory::classify("disk:sda", "used"),
            FactTtlCategory::Dynamic
        );
        assert_eq!(
            FactTtlCategory::classify("proc:1234", "status"),
            FactTtlCategory::Volatile
        );
    }

    #[test]
    fn test_ttl_durations() {
        assert_eq!(FactTtlCategory::Static.ttl(), Duration::days(7));
        assert_eq!(FactTtlCategory::Dynamic.ttl(), Duration::hours(1));
        assert_eq!(FactTtlCategory::Volatile.ttl(), Duration::minutes(5));
    }

    #[test]
    fn test_ttl_fact_creation() {
        let fact = TtlFact::new(
            "cpu:0".to_string(),
            "model".to_string(),
            "AMD Ryzen 9".to_string(),
            0.95,
            FactSourceV22::Probe {
                probe_id: "cpu.info".to_string(),
                timestamp: Utc::now(),
            },
        );

        assert_eq!(fact.ttl_category, FactTtlCategory::Static);
        assert!(!fact.is_expired());
        assert_eq!(fact.validation, ValidationStatus::Pending);
    }

    #[test]
    fn test_fact_brain_store_get() {
        let mut brain = FactBrain::new();

        let fact = TtlFact::new(
            "cpu:0".to_string(),
            "cores".to_string(),
            "8".to_string(),
            0.95,
            FactSourceV22::Probe {
                probe_id: "cpu.info".to_string(),
                timestamp: Utc::now(),
            },
        );

        brain.store(fact);

        let retrieved = brain.get("cpu:0", "cores");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "8");
    }

    #[test]
    fn test_fact_brain_prefix_query() {
        let mut brain = FactBrain::new();

        brain.store(TtlFact::new(
            "pkg:vim".to_string(),
            "version".to_string(),
            "9.0".to_string(),
            0.9,
            FactSourceV22::Probe {
                probe_id: "pkg.list".to_string(),
                timestamp: Utc::now(),
            },
        ));

        brain.store(TtlFact::new(
            "pkg:neovim".to_string(),
            "version".to_string(),
            "0.9".to_string(),
            0.9,
            FactSourceV22::Probe {
                probe_id: "pkg.list".to_string(),
                timestamp: Utc::now(),
            },
        ));

        let pkgs = brain.get_by_prefix("pkg:");
        assert_eq!(pkgs.len(), 2);
    }

    #[test]
    fn test_question_type_answerable() {
        assert!(QuestionType::FactLookup.answerable_from_facts());
        assert!(QuestionType::StatusCheck.answerable_from_facts());
        assert!(!QuestionType::Explanation.answerable_from_facts());
    }

    #[test]
    fn test_semantic_link() {
        let mut brain = FactBrain::new();

        brain.store(TtlFact::new(
            "cpu:0".to_string(),
            "cores".to_string(),
            "8".to_string(),
            0.95,
            FactSourceV22::Probe {
                probe_id: "cpu.info".to_string(),
                timestamp: Utc::now(),
            },
        ));

        brain.store(TtlFact::new(
            "cpu:0".to_string(),
            "model".to_string(),
            "AMD Ryzen 9".to_string(),
            0.95,
            FactSourceV22::Probe {
                probe_id: "cpu.info".to_string(),
                timestamp: Utc::now(),
            },
        ));

        brain.add_link(SemanticLink {
            from_fact: "cpu:0:cores".to_string(),
            to_fact: "cpu:0:model".to_string(),
            relation: SemanticRelation::SameEntity,
            strength: 1.0,
        });

        let related = brain.get_related("cpu:0", "cores");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].attribute, "model");
    }

    #[test]
    fn test_validation_status() {
        let mut fact = TtlFact::new(
            "test:entity".to_string(),
            "attr".to_string(),
            "value".to_string(),
            0.9,
            FactSourceV22::User { user_id: None },
        );

        assert_eq!(fact.validation, ValidationStatus::Pending);

        fact.mark_validated(ValidationResult::Valid);
        assert_eq!(fact.validation, ValidationStatus::Valid);
    }

    #[test]
    fn test_aggregate_ops() {
        let strategy = SynthesisStrategy::Aggregate {
            operation: AggregateOp::Sum,
            fact_keys: vec!["disk:sda:size".to_string(), "disk:sdb:size".to_string()],
        };

        let json = serde_json::to_string(&strategy).unwrap();
        assert!(json.contains("aggregate"));
        assert!(json.contains("sum"));
    }
}
