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
pub mod pipeline_mapping;
pub mod pipeline_state;
pub mod pipeline_types;
pub mod reasoning_prompt;
pub mod reasoning_types;
pub mod reasoning_validator;
pub mod translator;
pub mod translator_builder;
pub mod translator_helpers;
pub mod translator_types;

// Re-export main types
pub use evidence::{
    extract_blame, extract_boot_time, extract_disk, extract_failed_services, extract_memory,
    fact_domain, EvidenceBundle, EvidenceBundleBuilder, FactValue, ProbeError,
};
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
pub use reasoning_prompt::{
    build_reasoning_prompt, parse_reasoning_output, REASONING_SYSTEM_PROMPT,
};
pub use reasoning_types::{
    DerivedValues, ReasoningEvidence, ReasoningOutput, ReasoningRequest, MAX_REASONING_CHARS,
};
pub use reasoning_validator::{ReasoningQuality, ReasoningValidator};
pub use translator::{PrecisionTranslator, TranslatedAnswer, TranslationError};
pub use translator::{
    MAX_BOOLEAN_ANSWER, MAX_BRIEF_ANSWER, MAX_ENTITY_ANSWER, MAX_LIST_ITEMS, MAX_NUMERIC_ANSWER,
};
pub use translator_builder::DirectAnswerBuilder;
