//! ERA Pipeline - Evidence → Reasoning → Answer (v0.0.441).
//!
//! Universal pipeline replacing case-by-case specialist logic.
//!
//! Every request follows this pipeline:
//! 1) EVIDENCE - Collect facts deterministically
//! 2) REASONING - LLM reasons over evidence only
//! 3) ANSWER - Translator converts reasoning to precise answer
//!
//! No stage may skip the previous one.

pub mod evidence;
pub mod learning;
pub mod metrics;
pub mod pipeline;
pub mod reasoning;
pub mod translator;

// Re-export main types
pub use evidence::{EvidenceBundle, EvidenceBundleBuilder, FactValue, ProbeError};
pub use learning::{
    decide_fast_path, FastPathDecision, IntentFactMapping, IntentLearningStore, LearningStats,
};
pub use metrics::{
    validate_resolution, HonestMetrics, MetricsSummary, ResolutionCriteria, ResolutionReason,
    ResolutionStatus, DEFAULT_CONFIDENCE_THRESHOLD,
};
pub use pipeline::{
    AnswerType, EraPipeline, ExtractedIntent, FactProbeTable, PipelineError, PipelineStage,
    PipelineStatus, ProbeMapping,
};
pub use reasoning::{
    build_reasoning_prompt, parse_reasoning_output, DerivedValues, ReasoningEvidence,
    ReasoningOutput, ReasoningQuality, ReasoningRequest, ReasoningValidator,
    MAX_REASONING_CHARS, REASONING_SYSTEM_PROMPT,
};
pub use translator::{
    DirectAnswerBuilder, PrecisionTranslator, TranslatedAnswer, TranslationError,
    MAX_BOOLEAN_ANSWER, MAX_BRIEF_ANSWER, MAX_ENTITY_ANSWER, MAX_LIST_ITEMS, MAX_NUMERIC_ANSWER,
};
