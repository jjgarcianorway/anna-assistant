//! Protocol v0.21.0 - Hybrid Answer Pipeline
//!
//! Key principles:
//! - Fast-first: Answer from cached facts when possible
//! - Selective probing: Only probe what's truly needed
//! - No loops: Make decisions upfront, avoid iterations
//!
//! Pipeline:
//! 1. Fast path: Check if question can be answered from facts alone
//! 2. Gap analysis: Identify what's missing
//! 3. Targeted probing: Run only necessary probes
//! 4. Synthesis: Generate final answer

use serde::{Deserialize, Serialize};

/// Confidence levels for fast-path answers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastPathConfidence {
    /// High confidence from fresh facts (>90%)
    High,
    /// Medium confidence from slightly stale facts (70-90%)
    Medium,
    /// Low confidence, needs probing (<70%)
    Low,
    /// Cannot answer without probing
    NeedsProbing,
}

impl FastPathConfidence {
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::High,
            70..=89 => Self::Medium,
            1..=69 => Self::Low,
            0 => Self::NeedsProbing,
            _ => Self::NeedsProbing,
        }
    }

    pub fn can_answer_directly(&self) -> bool {
        matches!(self, Self::High | Self::Medium)
    }
}

/// Result of the fast-path fact check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathResult {
    /// Whether we can answer from facts alone
    pub can_answer: bool,
    /// Confidence level if we can answer
    pub confidence: FastPathConfidence,
    /// Relevant facts found
    pub relevant_facts: Vec<RelevantFact>,
    /// Gaps in knowledge (what's missing)
    pub knowledge_gaps: Vec<KnowledgeGap>,
    /// Suggested answer if confidence is high/medium
    pub suggested_answer: Option<String>,
    /// Reasoning for the assessment
    pub reasoning: String,
}

/// A relevant fact from the knowledge store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantFact {
    /// Fact entity (e.g., "cpu:0")
    pub entity: String,
    /// Fact attribute (e.g., "model")
    pub attribute: String,
    /// Fact value
    pub value: String,
    /// Trust score (0.0-1.0)
    pub trust: f32,
    /// How fresh the fact is
    pub freshness: FactFreshness,
}

/// Freshness level of a fact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactFreshness {
    /// Fresh (< 1 hour old)
    Fresh,
    /// Recent (1-24 hours old)
    Recent,
    /// Stale (> 24 hours old)
    Stale,
    /// Unknown age
    Unknown,
}

impl FactFreshness {
    pub fn from_age_hours(hours: i64) -> Self {
        match hours {
            0 => Self::Fresh,
            1..=24 => Self::Recent,
            _ => Self::Stale,
        }
    }
}

/// A gap in knowledge that needs to be filled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    /// What information is missing
    pub description: String,
    /// Probe that can fill this gap
    pub probe_id: String,
    /// Priority (1 = highest)
    pub priority: u8,
    /// Whether this gap blocks the answer
    pub blocking: bool,
}

/// Targeted probing request (only run what's needed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetedProbing {
    /// Probes to run (in priority order)
    pub probes: Vec<TargetedProbe>,
    /// Maximum probes to run
    pub max_probes: usize,
    /// Timeout per probe (ms)
    pub timeout_ms: u64,
}

impl Default for TargetedProbing {
    fn default() -> Self {
        Self {
            probes: Vec::new(),
            max_probes: 3,
            timeout_ms: 5000,
        }
    }
}

/// A probe to run with specific purpose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetedProbe {
    /// Probe ID
    pub probe_id: String,
    /// Why this probe is needed
    pub reason: String,
    /// What gap it fills
    pub fills_gap: String,
    /// Skip if this fact already exists
    pub skip_if_fact_exists: Option<String>,
}

/// Result of targeted probing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbingResult {
    /// Probes that were run
    pub probes_run: Vec<ProbeOutcome>,
    /// Probes that were skipped (fact already existed)
    pub probes_skipped: Vec<String>,
    /// New facts discovered
    pub new_facts: Vec<RelevantFact>,
    /// Total time taken (ms)
    pub total_time_ms: u64,
}

/// Outcome of a single probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub probe_id: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

/// Hybrid answer pipeline state
#[derive(Debug, Clone)]
pub struct HybridPipeline {
    /// Original question
    pub question: String,
    /// Fast-path result
    pub fast_path: Option<FastPathResult>,
    /// Targeted probing (if needed)
    pub probing: Option<TargetedProbing>,
    /// Probing result (if probing was done)
    pub probing_result: Option<ProbingResult>,
    /// Final answer
    pub final_answer: Option<HybridAnswer>,
    /// Pipeline stage
    pub stage: PipelineStage,
}

impl HybridPipeline {
    pub fn new(question: String) -> Self {
        Self {
            question,
            fast_path: None,
            probing: None,
            probing_result: None,
            final_answer: None,
            stage: PipelineStage::FastPath,
        }
    }

    /// Advance to the next stage
    pub fn advance(&mut self) {
        self.stage = match self.stage {
            PipelineStage::FastPath => {
                if let Some(ref fp) = self.fast_path {
                    if fp.can_answer {
                        PipelineStage::Synthesis
                    } else {
                        PipelineStage::GapAnalysis
                    }
                } else {
                    PipelineStage::GapAnalysis
                }
            }
            PipelineStage::GapAnalysis => PipelineStage::TargetedProbing,
            PipelineStage::TargetedProbing => PipelineStage::Synthesis,
            PipelineStage::Synthesis => PipelineStage::Complete,
            PipelineStage::Complete => PipelineStage::Complete,
        };
    }

    /// Check if pipeline is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.stage, PipelineStage::Complete)
    }
}

/// Pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    /// Checking facts for fast-path answer
    FastPath,
    /// Analyzing what knowledge gaps exist
    GapAnalysis,
    /// Running targeted probes
    TargetedProbing,
    /// Synthesizing final answer
    Synthesis,
    /// Pipeline complete
    Complete,
}

impl Default for PipelineStage {
    fn default() -> Self {
        Self::FastPath
    }
}

/// The final hybrid answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridAnswer {
    /// Answer text
    pub text: String,
    /// How the answer was generated
    pub source: AnswerSource,
    /// Reliability score (0-100)
    pub reliability: u8,
    /// Reliability explanation
    pub reliability_note: String,
    /// Facts used
    pub facts_used: Vec<String>,
    /// Probes run (if any)
    pub probes_run: Vec<String>,
    /// Total pipeline time (ms)
    pub total_time_ms: u64,
}

/// How the answer was generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerSource {
    /// Answered entirely from cached facts (fast path)
    CachedFacts,
    /// Answered from facts with minor probing
    FactsWithProbing,
    /// Answered primarily from probing
    ProbingBased,
    /// Partial answer (some gaps remain)
    Partial,
}

impl AnswerSource {
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::CachedFacts => "[F]",
            Self::FactsWithProbing => "[?]",
            Self::ProbingBased => "[P]",
            Self::Partial => "[!]",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::CachedFacts => "Answered from cached knowledge (instant)",
            Self::FactsWithProbing => "Answered with targeted probing",
            Self::ProbingBased => "Answered from fresh probing",
            Self::Partial => "Partial answer (some data unavailable)",
        }
    }
}

/// Junior's action in v0.21.0
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum JuniorActionV21 {
    /// Perform fast-path check
    #[serde(rename = "fast_path")]
    FastPath { result: FastPathResult },

    /// Plan targeted probing
    #[serde(rename = "plan_probing")]
    PlanProbing { probing: TargetedProbing },

    /// Synthesize answer (from facts or after probing)
    #[serde(rename = "synthesize")]
    Synthesize { answer: HybridAnswer },
}

/// Senior's review in v0.21.0 (simplified - no loops)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "review")]
pub enum SeniorReviewV21 {
    /// Approve the fast-path assessment
    #[serde(rename = "approve_fast_path")]
    ApproveFastPath { feedback: String },

    /// Override fast-path (force probing)
    #[serde(rename = "force_probing")]
    ForceProbing {
        reason: String,
        additional_probes: Vec<String>,
    },

    /// Approve the final answer
    #[serde(rename = "approve_answer")]
    ApproveAnswer { reliability: u8, note: String },

    /// Amend the final answer
    #[serde(rename = "amend_answer")]
    AmendAnswer {
        amended_text: String,
        reliability: u8,
        note: String,
    },
}

/// Configuration for hybrid pipeline
#[derive(Debug, Clone)]
pub struct HybridConfig {
    /// Minimum confidence for fast-path (0-100)
    pub fast_path_min_confidence: u8,
    /// Maximum probes per question
    pub max_probes: usize,
    /// Probe timeout (ms)
    pub probe_timeout_ms: u64,
    /// Skip Senior review for high-confidence fast-path
    pub skip_senior_for_fast_path: bool,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            fast_path_min_confidence: 80,
            max_probes: 3,
            probe_timeout_ms: 5000,
            skip_senior_for_fast_path: true,
        }
    }
}

/// Constants for v0.21.0
pub const MAX_PROBES_V21: usize = 3;
pub const FAST_PATH_MIN_CONFIDENCE: u8 = 80;
pub const PROBE_TIMEOUT_MS: u64 = 5000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_from_score() {
        assert_eq!(FastPathConfidence::from_score(95), FastPathConfidence::High);
        assert_eq!(FastPathConfidence::from_score(85), FastPathConfidence::Medium);
        assert_eq!(FastPathConfidence::from_score(50), FastPathConfidence::Low);
        assert_eq!(FastPathConfidence::from_score(0), FastPathConfidence::NeedsProbing);
    }

    #[test]
    fn test_confidence_can_answer() {
        assert!(FastPathConfidence::High.can_answer_directly());
        assert!(FastPathConfidence::Medium.can_answer_directly());
        assert!(!FastPathConfidence::Low.can_answer_directly());
        assert!(!FastPathConfidence::NeedsProbing.can_answer_directly());
    }

    #[test]
    fn test_fact_freshness_from_age() {
        assert_eq!(FactFreshness::from_age_hours(0), FactFreshness::Fresh);
        assert_eq!(FactFreshness::from_age_hours(12), FactFreshness::Recent);
        assert_eq!(FactFreshness::from_age_hours(48), FactFreshness::Stale);
    }

    #[test]
    fn test_pipeline_stages() {
        let mut pipeline = HybridPipeline::new("What is my CPU?".to_string());
        assert_eq!(pipeline.stage, PipelineStage::FastPath);

        // No fast path result yet, advances to gap analysis
        pipeline.advance();
        assert_eq!(pipeline.stage, PipelineStage::GapAnalysis);

        pipeline.advance();
        assert_eq!(pipeline.stage, PipelineStage::TargetedProbing);

        pipeline.advance();
        assert_eq!(pipeline.stage, PipelineStage::Synthesis);

        pipeline.advance();
        assert!(pipeline.is_complete());
    }

    #[test]
    fn test_fast_path_skips_to_synthesis() {
        let mut pipeline = HybridPipeline::new("What is my CPU?".to_string());
        pipeline.fast_path = Some(FastPathResult {
            can_answer: true,
            confidence: FastPathConfidence::High,
            relevant_facts: vec![],
            knowledge_gaps: vec![],
            suggested_answer: Some("AMD Ryzen 9".to_string()),
            reasoning: "Fresh CPU facts available".to_string(),
        });

        pipeline.advance();
        // Should skip directly to synthesis
        assert_eq!(pipeline.stage, PipelineStage::Synthesis);
    }

    #[test]
    fn test_answer_source_descriptions() {
        assert_eq!(AnswerSource::CachedFacts.indicator(), "[F]");
        assert!(AnswerSource::CachedFacts.description().contains("instant"));
    }

    #[test]
    fn test_serialize_junior_action() {
        let action = JuniorActionV21::FastPath {
            result: FastPathResult {
                can_answer: true,
                confidence: FastPathConfidence::High,
                relevant_facts: vec![],
                knowledge_gaps: vec![],
                suggested_answer: Some("Test answer".to_string()),
                reasoning: "Test reason".to_string(),
            },
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("fast_path"));
    }

    #[test]
    fn test_default_hybrid_config() {
        let config = HybridConfig::default();
        assert_eq!(config.fast_path_min_confidence, 80);
        assert_eq!(config.max_probes, 3);
        assert!(config.skip_senior_for_fast_path);
    }
}
