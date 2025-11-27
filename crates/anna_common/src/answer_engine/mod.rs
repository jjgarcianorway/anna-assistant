//! Answer Engine
//!
//! v0.14.0: Aligned to Reality (6 real probes)
//! v0.15.0: Junior/Senior architecture with dynamic checks
//!
//! Features:
//! - Deterministic probe usage
//! - Explicit evidence citations
//! - Supervised LLM-A/LLM-B loop
//! - Transparent reliability scoring
//! - Partial answer fallback
//! - v0.15.0: Risk classification, user questions, learning

pub mod evidence;
pub mod protocol;
pub mod protocol_v15;
pub mod scoring;

pub use evidence::*;
pub use protocol::*;
pub use scoring::*;

// v0.15.0 protocol types (next iteration)
pub use protocol_v15::{
    CheckApproval, CheckRequest, CheckResult, CheckRisk, CheckRiskEval, CoreProbeId,
    CoreProbeInfo, DynamicCheck, FactSource, Intent, LearningUpdate, LlmARequestV15,
    LlmAResponseV15, LlmBRequestV15, LlmBResponseV15, LlmBVerdict, QuestionOption,
    QuestionStyle, ReasoningTrace, TraceStep, TrackedFact, UserAnswer, UserAnswerValue,
    UserQuestion, PROTOCOL_VERSION as PROTOCOL_VERSION_V15,
};
