//! Faster Answers Module - v0.24.0
//!
//! Question classification, answer caching, and fast-path strategies.
//! Zero hardcoded knowledge - all classification is LLM-derived.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// Question Classification
// ============================================================================

/// Question type classification (LLM-derived)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum QuestionType {
    /// Simple factual question (e.g., "How many CPU cores?")
    Factual,
    /// Status/state query (e.g., "Is nginx running?")
    Status,
    /// How-to question (e.g., "How do I install X?")
    HowTo,
    /// Configuration question (e.g., "What's my shell?")
    Configuration,
    /// Troubleshooting (e.g., "Why is X slow?")
    Troubleshooting,
    /// Comparison (e.g., "Which is better, X or Y?")
    Comparison,
    /// List/enumerate (e.g., "What packages are installed?")
    Enumeration,
    /// Definition (e.g., "What is systemd?")
    Definition,
    /// Yes/No question
    YesNo,
    /// Complex/multi-part question
    Complex,
    /// Unknown/unclassified
    Unknown,
}

impl QuestionType {
    /// Get the typical complexity score for this type (1-10)
    pub fn typical_complexity(&self) -> u8 {
        match self {
            Self::Factual => 2,
            Self::Status => 2,
            Self::YesNo => 2,
            Self::Configuration => 3,
            Self::Definition => 3,
            Self::Enumeration => 4,
            Self::HowTo => 5,
            Self::Comparison => 6,
            Self::Troubleshooting => 7,
            Self::Complex => 8,
            Self::Unknown => 5,
        }
    }

    /// Can this type typically be answered from cache?
    pub fn is_cacheable(&self) -> bool {
        matches!(
            self,
            Self::Factual | Self::Definition | Self::Configuration
        )
    }

    /// Does this type require fresh probes?
    pub fn requires_fresh_data(&self) -> bool {
        matches!(
            self,
            Self::Status | Self::Troubleshooting | Self::Enumeration
        )
    }
}

/// Classified question with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedQuestion {
    /// Original question text
    pub text: String,
    /// Normalized/cleaned question
    pub normalized: String,
    /// Detected question type
    pub question_type: QuestionType,
    /// Complexity score (1-10)
    pub complexity: u8,
    /// Keywords extracted
    pub keywords: Vec<String>,
    /// Entities detected (packages, services, paths)
    pub entities: Vec<DetectedEntity>,
    /// Suggested answer strategy
    pub strategy: AnswerStrategy,
    /// Confidence in classification (0.0-1.0)
    pub confidence: f32,
    /// Classification timestamp
    pub classified_at: i64,
}

/// Entity detected in a question
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectedEntity {
    /// Entity text
    pub text: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Start position in question
    pub start: usize,
    /// End position in question
    pub end: usize,
}

/// Types of entities we can detect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Package,
    Service,
    FilePath,
    Command,
    User,
    Group,
    Port,
    IpAddress,
    Hostname,
    ProcessName,
    DevicePath,
    ConfigKey,
    EnvironmentVar,
    Unknown,
}

// ============================================================================
// Answer Strategy
// ============================================================================

/// Strategy for answering a question
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnswerStrategy {
    /// Answer directly from cache
    CacheHit,
    /// Answer from fact store without probing
    FactStoreOnly,
    /// Quick single-probe answer
    SingleProbe(String),
    /// Multi-probe with specific probes
    MultiProbe(Vec<String>),
    /// Requires LLM reasoning
    LlmRequired,
    /// Full research loop needed
    FullResearch,
    /// Question cannot be answered
    CannotAnswer(String),
}

impl AnswerStrategy {
    /// Get the typical time to answer for this strategy
    pub fn typical_latency(&self) -> Duration {
        match self {
            Self::CacheHit => Duration::from_millis(10),
            Self::FactStoreOnly => Duration::from_millis(50),
            Self::SingleProbe(_) => Duration::from_millis(200),
            Self::MultiProbe(probes) => Duration::from_millis(200 * probes.len() as u64),
            Self::LlmRequired => Duration::from_secs(2),
            Self::FullResearch => Duration::from_secs(10),
            Self::CannotAnswer(_) => Duration::from_millis(100),
        }
    }
}

// ============================================================================
// Answer Cache
// ============================================================================

/// Cached answer entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnswer {
    /// Cache key (normalized question hash)
    pub key: String,
    /// Original question
    pub question: String,
    /// The answer
    pub answer: String,
    /// Confidence score
    pub confidence: f32,
    /// When this was cached
    pub cached_at: i64,
    /// When this expires
    pub expires_at: i64,
    /// How many times this was served
    pub hit_count: u64,
    /// Last time this was served
    pub last_hit_at: Option<i64>,
    /// Question type for this answer
    pub question_type: QuestionType,
    /// Fact keys this answer depends on
    pub depends_on_facts: Vec<String>,
}

impl CachedAnswer {
    /// Check if this cache entry is still valid
    pub fn is_valid(&self, now: i64) -> bool {
        now < self.expires_at
    }

    /// Get remaining TTL in seconds
    pub fn ttl_seconds(&self, now: i64) -> i64 {
        self.expires_at - now
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache entries
    pub max_entries: usize,
    /// Default TTL for cached answers
    pub default_ttl_seconds: u64,
    /// TTL for factual answers
    pub factual_ttl_seconds: u64,
    /// TTL for status answers (shorter)
    pub status_ttl_seconds: u64,
    /// TTL for configuration answers
    pub config_ttl_seconds: u64,
    /// Enable cache persistence
    pub persist_to_disk: bool,
    /// Cache file path
    pub cache_path: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl_seconds: 3600,       // 1 hour
            factual_ttl_seconds: 86400,      // 24 hours
            status_ttl_seconds: 60,          // 1 minute
            config_ttl_seconds: 3600,        // 1 hour
            persist_to_disk: true,
            cache_path: None,
        }
    }
}

impl CacheConfig {
    /// Get TTL for a question type
    pub fn ttl_for_type(&self, question_type: &QuestionType) -> u64 {
        match question_type {
            QuestionType::Factual | QuestionType::Definition => self.factual_ttl_seconds,
            QuestionType::Status => self.status_ttl_seconds,
            QuestionType::Configuration => self.config_ttl_seconds,
            _ => self.default_ttl_seconds,
        }
    }
}

/// Answer cache state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerCache {
    /// Cached answers by key
    pub entries: HashMap<String, CachedAnswer>,
    /// LRU order (oldest first)
    pub lru_order: Vec<String>,
    /// Cache statistics
    pub stats: CacheStatistics,
    /// Configuration
    pub config: CacheConfig,
}

impl Default for AnswerCache {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            lru_order: Vec::new(),
            stats: CacheStatistics::default(),
            config: CacheConfig::default(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStatistics {
    /// Total lookups
    pub total_lookups: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Entries evicted
    pub evictions: u64,
    /// TTL expirations
    pub expirations: u64,
    /// Total bytes saved (approximate)
    pub bytes_saved: u64,
}

impl CacheStatistics {
    /// Calculate hit rate
    pub fn hit_rate(&self) -> f32 {
        if self.total_lookups == 0 {
            return 0.0;
        }
        (self.hits as f32 / self.total_lookups as f32) * 100.0
    }
}

// ============================================================================
// Fast-Path Detection
// ============================================================================

/// Fast-path eligibility result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathResult {
    /// Is this eligible for fast path?
    pub eligible: bool,
    /// Which fast path to use
    pub path: Option<FastPath>,
    /// Reason if not eligible
    pub reason: Option<String>,
    /// Estimated time savings in ms
    pub estimated_savings_ms: u64,
}

/// Types of fast paths
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FastPath {
    /// Direct cache hit
    Cache,
    /// Fact store lookup
    FactStore,
    /// Single probe (named)
    SingleProbe(String),
    /// Static answer (help, version)
    Static,
}

/// Question patterns for fast-path detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionPattern {
    /// Pattern ID
    pub id: String,
    /// Regex pattern for matching
    pub pattern: String,
    /// Suggested fast path
    pub fast_path: FastPath,
    /// Required fact keys
    pub required_facts: Vec<String>,
    /// Confidence threshold for using this pattern
    pub confidence_threshold: f32,
}

// ============================================================================
// Answer Pipeline Configuration
// ============================================================================

/// Configuration for the fast answer pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastAnswerConfig {
    /// Enable caching
    pub enable_cache: bool,
    /// Enable fact store fast path
    pub enable_fact_store: bool,
    /// Enable question classification
    pub enable_classification: bool,
    /// Maximum time for fast path attempts (ms)
    pub fast_path_timeout_ms: u64,
    /// Minimum confidence to use cache
    pub cache_confidence_threshold: f32,
    /// Question patterns for fast path
    pub patterns: Vec<QuestionPattern>,
    /// Cache configuration
    pub cache_config: CacheConfig,
}

impl Default for FastAnswerConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            enable_fact_store: true,
            enable_classification: true,
            fast_path_timeout_ms: 500,
            cache_confidence_threshold: 0.8,
            patterns: Vec::new(),
            cache_config: CacheConfig::default(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_type_complexity() {
        assert_eq!(QuestionType::Factual.typical_complexity(), 2);
        assert_eq!(QuestionType::Complex.typical_complexity(), 8);
        assert_eq!(QuestionType::Troubleshooting.typical_complexity(), 7);
    }

    #[test]
    fn test_question_type_cacheable() {
        assert!(QuestionType::Factual.is_cacheable());
        assert!(QuestionType::Definition.is_cacheable());
        assert!(!QuestionType::Status.is_cacheable());
        assert!(!QuestionType::Troubleshooting.is_cacheable());
    }

    #[test]
    fn test_answer_strategy_latency() {
        assert!(AnswerStrategy::CacheHit.typical_latency() < Duration::from_millis(100));
        assert!(AnswerStrategy::FullResearch.typical_latency() > Duration::from_secs(5));
    }

    #[test]
    fn test_cached_answer_validity() {
        let answer = CachedAnswer {
            key: "test".to_string(),
            question: "Test?".to_string(),
            answer: "Yes".to_string(),
            confidence: 0.9,
            cached_at: 1000,
            expires_at: 2000,
            hit_count: 0,
            last_hit_at: None,
            question_type: QuestionType::Factual,
            depends_on_facts: vec![],
        };

        assert!(answer.is_valid(1500));
        assert!(!answer.is_valid(2500));
        assert_eq!(answer.ttl_seconds(1500), 500);
    }

    #[test]
    fn test_cache_config_ttl() {
        let config = CacheConfig::default();
        assert_eq!(
            config.ttl_for_type(&QuestionType::Factual),
            config.factual_ttl_seconds
        );
        assert_eq!(
            config.ttl_for_type(&QuestionType::Status),
            config.status_ttl_seconds
        );
    }

    #[test]
    fn test_cache_statistics_hit_rate() {
        let stats = CacheStatistics {
            total_lookups: 100,
            hits: 80,
            misses: 20,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_classified_question_serialize() {
        let classified = ClassifiedQuestion {
            text: "How many CPU cores do I have?".to_string(),
            normalized: "cpu cores count".to_string(),
            question_type: QuestionType::Factual,
            complexity: 2,
            keywords: vec!["cpu".to_string(), "cores".to_string()],
            entities: vec![],
            strategy: AnswerStrategy::SingleProbe("lscpu".to_string()),
            confidence: 0.95,
            classified_at: 1700000000,
        };

        let json = serde_json::to_string(&classified).unwrap();
        assert!(json.contains("Factual"));

        let parsed: ClassifiedQuestion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.question_type, QuestionType::Factual);
    }

    #[test]
    fn test_detected_entity() {
        let entity = DetectedEntity {
            text: "nginx".to_string(),
            entity_type: EntityType::Service,
            start: 5,
            end: 10,
        };
        assert_eq!(entity.entity_type, EntityType::Service);
    }

    #[test]
    fn test_fast_answer_config_default() {
        let config = FastAnswerConfig::default();
        assert!(config.enable_cache);
        assert!(config.enable_fact_store);
        assert_eq!(config.fast_path_timeout_ms, 500);
    }
}
