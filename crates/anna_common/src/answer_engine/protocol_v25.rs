//! Protocol v0.25.0: Relevance First, Idle Learning, No Hard-Coding
//!
//! This module implements:
//! - Discover-first approach (no hardcoding)
//! - Usage ranking and active session awareness
//! - Ambiguity handling with user clarification
//! - Relevance-based answer strategies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol version identifier
pub const PROTOCOL_VERSION_V25: &str = "0.25.0";

// ============================================================================
// RELEVANCE SCORING
// ============================================================================

/// Relevance score for an entity (app, fact, probe result)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceScore {
    /// Overall score (0.0-1.0)
    pub score: f32,
    /// Individual factors contributing to score
    pub factors: RelevanceFactors,
    /// When this score was calculated
    pub calculated_at: i64,
    /// Whether this score is stale and needs refresh
    pub is_stale: bool,
}

impl RelevanceScore {
    /// Create a new relevance score
    pub fn new(score: f32, factors: RelevanceFactors) -> Self {
        Self {
            score: score.clamp(0.0, 1.0),
            factors,
            calculated_at: chrono::Utc::now().timestamp(),
            is_stale: false,
        }
    }

    /// Check if this score should be refreshed (older than max_age_secs)
    pub fn should_refresh(&self, max_age_secs: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.is_stale || (now - self.calculated_at) > max_age_secs
    }

    /// Mark as stale
    pub fn mark_stale(&mut self) {
        self.is_stale = true;
    }
}

/// Factors contributing to relevance score
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelevanceFactors {
    /// Recency factor (recent usage = higher)
    pub recency: f32,
    /// Frequency factor (frequent usage = higher)
    pub frequency: f32,
    /// Session context factor (matches current session = higher)
    pub session_context: f32,
    /// User preference factor (explicit preference = higher)
    pub user_preference: f32,
    /// Discovered via probes (not hardcoded) = higher
    pub discovery_bonus: f32,
}

impl RelevanceFactors {
    /// Calculate weighted total
    pub fn weighted_total(&self) -> f32 {
        // Weights for each factor
        const W_RECENCY: f32 = 0.25;
        const W_FREQUENCY: f32 = 0.25;
        const W_SESSION: f32 = 0.20;
        const W_USER_PREF: f32 = 0.20;
        const W_DISCOVERY: f32 = 0.10;

        (self.recency * W_RECENCY)
            + (self.frequency * W_FREQUENCY)
            + (self.session_context * W_SESSION)
            + (self.user_preference * W_USER_PREF)
            + (self.discovery_bonus * W_DISCOVERY)
    }
}

// ============================================================================
// USAGE EVENTS
// ============================================================================

/// Type of usage event
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageEventType {
    /// App was launched
    AppLaunch,
    /// App gained focus
    AppFocus,
    /// App was closed
    AppClose,
    /// File was accessed
    FileAccess,
    /// Command was executed
    CommandExec,
    /// Probe was successful
    ProbeSuccess,
    /// User asked about something
    UserQuery,
    /// Answer was provided for topic
    AnswerProvided,
}

/// A usage event recorded by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: UsageEventType,
    /// Entity involved (app name, path, command, etc.)
    pub entity: String,
    /// Additional context
    pub context: HashMap<String, String>,
    /// Timestamp when event occurred
    pub timestamp: i64,
    /// Session ID when this happened
    pub session_id: Option<String>,
}

impl UsageEvent {
    /// Create a new usage event
    pub fn new(event_type: UsageEventType, entity: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            entity,
            context: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
            session_id: None,
        }
    }

    /// Add context
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

// ============================================================================
// ENTITY RANKING
// ============================================================================

/// A ranked entity (app, service, fact, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedEntity {
    /// Entity identifier
    pub id: String,
    /// Entity type
    pub entity_type: RankedEntityType,
    /// Display name
    pub display_name: String,
    /// Relevance score
    pub relevance: RelevanceScore,
    /// Usage count in current period
    pub usage_count: u64,
    /// Last used timestamp
    pub last_used: Option<i64>,
    /// Discovery source (probe, file scan, etc.)
    pub discovered_via: DiscoverySource,
}

/// Type of ranked entity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankedEntityType {
    Application,
    Service,
    Package,
    ConfigFile,
    Fact,
    Command,
    Path,
}

/// How an entity was discovered
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
    /// Discovered via probe execution
    Probe { probe_id: String },
    /// Discovered via file scan
    FileScan { path: String },
    /// Discovered via desktop session detection
    SessionDetection,
    /// User explicitly mentioned/configured
    UserInput,
    /// Inferred from other facts
    Inference { from: Vec<String> },
}

// ============================================================================
// AMBIGUITY HANDLING
// ============================================================================

/// Detected ambiguity in user query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAmbiguity {
    /// Ambiguity ID
    pub id: String,
    /// Type of ambiguity
    pub ambiguity_type: AmbiguityType,
    /// The ambiguous term/phrase
    pub term: String,
    /// Possible interpretations
    pub candidates: Vec<AmbiguityCandidate>,
    /// Confidence that this is actually ambiguous (0.0-1.0)
    pub confidence: f32,
    /// Context from the query
    pub query_context: String,
}

/// Type of ambiguity detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbiguityType {
    /// Multiple apps with similar names
    MultipleApps,
    /// Multiple packages match
    MultiplePackages,
    /// Multiple services match
    MultipleServices,
    /// Term has multiple meanings
    PolysemousTerm,
    /// Unclear scope (system vs user)
    UnclearScope,
    /// Unspecified version/variant
    UnspecifiedVariant,
}

/// A candidate interpretation for an ambiguous term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityCandidate {
    /// Candidate entity
    pub entity: RankedEntity,
    /// Confidence this is what user meant (0.0-1.0)
    pub confidence: f32,
    /// Reasoning for this candidate
    pub reasoning: String,
}

/// User's resolution of ambiguity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityResolution {
    /// Original ambiguity ID
    pub ambiguity_id: String,
    /// Selected candidate index
    pub selected_index: usize,
    /// Whether to remember for future
    pub remember: bool,
    /// Timestamp
    pub resolved_at: i64,
}

// ============================================================================
// ANSWER STRATEGIES (RELEVANCE-AWARE)
// ============================================================================

/// Relevance-aware answer strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantAnswerStrategy {
    /// Base strategy to use
    pub strategy: AnswerPath,
    /// Relevant facts to include
    pub relevant_facts: Vec<String>,
    /// Ranked entities involved
    pub ranked_entities: Vec<RankedEntity>,
    /// Ambiguities to resolve (if any)
    pub ambiguities: Vec<DetectedAmbiguity>,
    /// Estimated answer quality (0.0-1.0)
    pub estimated_quality: f32,
    /// Whether user clarification is needed
    pub needs_clarification: bool,
}

/// Path to answer a question
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerPath {
    /// Answer from cached/known facts
    FromFacts,
    /// Single probe needed
    SingleProbe { probe_id: String },
    /// Multiple probes needed
    MultiProbe { probe_ids: Vec<String> },
    /// Need to resolve ambiguity first
    ResolveAmbiguity { ambiguity_id: String },
    /// Full research loop required
    FullResearch,
    /// Cannot answer
    CannotAnswer { reason: String },
}

// ============================================================================
// RELEVANCE-GUIDED MISSIONS
// ============================================================================

/// A learning mission guided by relevance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantMission {
    /// Mission ID
    pub id: String,
    /// Target fact keys to discover
    pub target_facts: Vec<String>,
    /// Relevance score for this mission
    pub relevance: f32,
    /// Why this mission is relevant
    pub relevance_reason: String,
    /// Probes to execute
    pub probes: Vec<String>,
    /// Priority (based on relevance + recency of need)
    pub priority: u8,
    /// Created timestamp
    pub created_at: i64,
}

impl RelevantMission {
    /// Create a new relevance-guided mission
    pub fn new(
        target_facts: Vec<String>,
        relevance: f32,
        reason: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            target_facts,
            relevance,
            relevance_reason: reason.to_string(),
            probes: Vec::new(),
            priority: ((1.0 - relevance) * 100.0) as u8, // Higher relevance = lower priority number
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Add probes
    pub fn with_probes(mut self, probes: Vec<String>) -> Self {
        self.probes = probes;
        self
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_factors() {
        let factors = RelevanceFactors {
            recency: 0.8,
            frequency: 0.6,
            session_context: 1.0,
            user_preference: 0.5,
            discovery_bonus: 1.0,
        };
        let total = factors.weighted_total();
        assert!(total > 0.0 && total <= 1.0);
    }

    #[test]
    fn test_relevance_score() {
        let factors = RelevanceFactors::default();
        let score = RelevanceScore::new(0.75, factors);
        assert_eq!(score.score, 0.75);
        assert!(!score.is_stale);
    }

    #[test]
    fn test_usage_event() {
        let event = UsageEvent::new(UsageEventType::AppLaunch, "firefox".to_string())
            .with_context("pid", "12345")
            .with_session("session-001".to_string());

        assert_eq!(event.event_type, UsageEventType::AppLaunch);
        assert_eq!(event.entity, "firefox");
        assert!(event.context.contains_key("pid"));
        assert!(event.session_id.is_some());
    }

    #[test]
    fn test_ambiguity_types() {
        let ambiguity = DetectedAmbiguity {
            id: "amb-001".to_string(),
            ambiguity_type: AmbiguityType::MultipleApps,
            term: "code".to_string(),
            candidates: vec![],
            confidence: 0.9,
            query_context: "open code".to_string(),
        };
        assert_eq!(ambiguity.ambiguity_type, AmbiguityType::MultipleApps);
    }

    #[test]
    fn test_relevant_mission() {
        let mission = RelevantMission::new(
            vec!["system.apps.browsers".to_string()],
            0.9,
            "User asked about browsers",
        ).with_probes(vec!["xdg_mime_query".to_string()]);

        assert_eq!(mission.relevance, 0.9);
        assert_eq!(mission.priority, 10); // High relevance = low priority number
    }

    #[test]
    fn test_answer_path_serialization() {
        let path = AnswerPath::SingleProbe { probe_id: "lscpu".to_string() };
        let json = serde_json::to_string(&path).unwrap();
        assert!(json.contains("single_probe"));

        let parsed: AnswerPath = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, path);
    }
}
