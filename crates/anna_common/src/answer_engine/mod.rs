//! Answer Engine
//!
//! v0.14.0: Aligned to Reality (6 real probes)
//! v0.15.0: Junior/Senior architecture with dynamic checks
//! v0.18.0: Step-by-step orchestration (one action per iteration)
//! v0.19.0: Subproblem decomposition, fact-aware planning, Senior as mentor
//! v0.21.0: Hybrid answer pipeline (fast-first, selective probing, no loops)
//! v0.22.0: Fact Brain & Question Decomposition (TTLs, validated facts)
//!
//! Features:
//! - Deterministic probe usage
//! - Explicit evidence citations
//! - Supervised LLM-A/LLM-B loop
//! - Transparent reliability scoring
//! - Partial answer fallback
//! - v0.15.0: Risk classification, user questions, learning
//! - v0.18.0: One probe per iteration, clear Junior/Senior roles
//! - v0.19.0: Subproblem decomposition, mentor-style Senior
//! - v0.21.0: Fast-first from facts, selective probing, no iteration loops
//! - v0.22.0: TTL-based facts, question decomposition, semantic linking

pub mod evidence;
pub mod protocol;
pub mod protocol_v15;
pub mod protocol_v18;
pub mod protocol_v19;
pub mod protocol_v21;
pub mod protocol_v22;
pub mod scoring;

pub use evidence::*;
pub use protocol::*;
pub use scoring::*;

// v0.15.0 protocol types (legacy)
pub use protocol_v15::{
    AnswerContinueRequest, AnswerSessionResponse, AnswerStartRequest, CheckApproval, CheckRequest,
    CheckResult, CheckRisk, CheckRiskEval, ConfirmCommandRequest, CoreProbeId, CoreProbeInfo,
    DynamicCheck, FactSource, Intent, LearningUpdate, LlmARequestV15, LlmAResponseV15,
    LlmBRequestV15, LlmBResponseV15, LlmBVerdict, QuestionOption, QuestionStyle, ReasoningTrace,
    TraceStep, TrackedFact, UserAnswer, UserAnswerValue, UserQuestion,
    PROTOCOL_VERSION as PROTOCOL_VERSION_V15,
};

// v0.18.0 protocol types (legacy)
pub use protocol_v18::{
    Clarification, EscalationRequest, EscalationSummary, FinalAnswerV18, HistoryEntry,
    JuniorDraft, JuniorRequest, JuniorScores, JuniorStep, LoopStatus, ProbeResultV18,
    QuestionLoopState, SeniorRequest, SeniorResponse, SeniorScores, MAX_ITERATIONS,
    MIN_SCORE_WITHOUT_SENIOR, PROTOCOL_VERSION as PROTOCOL_VERSION_V18, SCORE_GREEN, SCORE_YELLOW,
};

// v0.19.0 protocol types (legacy)
pub use protocol_v19::{
    FinalAnswerV19, JuniorDecomposition, JuniorScoresV19, JuniorStepV19, KnownFact, MentorContext,
    ProbeResultV19, QuestionStateV19, SeniorMentor, SeniorScoresV19, Subproblem, SubproblemMerge,
    SubproblemStatus, SubproblemSummary, SuggestedSubproblem, MAX_ITERATIONS_V19, MAX_SUBPROBLEMS,
    MIN_CONFIDENCE_FOR_SYNTHESIS,
};

// v0.21.0 protocol types (legacy)
pub use protocol_v21::{
    AnswerSource, FactFreshness, FastPathConfidence, FastPathResult, HybridAnswer, HybridConfig,
    HybridPipeline, JuniorActionV21, KnowledgeGap, PipelineStage, ProbeOutcome, ProbingResult,
    RelevantFact, SeniorReviewV21, TargetedProbe, TargetedProbing, FAST_PATH_MIN_CONFIDENCE,
    MAX_PROBES_V21, PROBE_TIMEOUT_MS,
};

// v0.22.0 protocol types (current)
pub use protocol_v22::{
    AggregateOp, AnalyzeQuestionRequest, AnalyzeQuestionResponse, FactBrain, FactSourceV22,
    FactTtlCategory, QuestionDecomposition, QuestionType, RequiredFact, SemanticLink,
    SemanticRelation, Subquestion, SynthesisStrategy, TtlFact, ValidationResult, ValidationStatus,
    DEFAULT_FACT_CONFIDENCE, MAX_SUBQUESTIONS, MIN_LINK_STRENGTH,
};
