//! Answer Engine
//!
//! v0.14.0: Aligned to Reality (6 real probes)
//! v0.15.0: Junior/Senior architecture with dynamic checks
//! v0.18.0: Step-by-step orchestration (one action per iteration)
//!
//! Features:
//! - Deterministic probe usage
//! - Explicit evidence citations
//! - Supervised LLM-A/LLM-B loop
//! - Transparent reliability scoring
//! - Partial answer fallback
//! - v0.15.0: Risk classification, user questions, learning
//! - v0.18.0: One probe per iteration, clear Junior/Senior roles

pub mod evidence;
pub mod protocol;
pub mod protocol_v15;
pub mod protocol_v18;
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

// v0.18.0 protocol types (current)
pub use protocol_v18::{
    Clarification, EscalationRequest, EscalationSummary, FinalAnswerV18, HistoryEntry,
    JuniorDraft, JuniorRequest, JuniorScores, JuniorStep, LoopStatus, ProbeResultV18,
    QuestionLoopState, SeniorRequest, SeniorResponse, SeniorScores, MAX_ITERATIONS,
    MIN_SCORE_WITHOUT_SENIOR, PROTOCOL_VERSION as PROTOCOL_VERSION_V18, SCORE_GREEN, SCORE_YELLOW,
};
