//! Diagnosis Conclusion Requirements (Part E) - v0.0.437.
//!
//! For category=diagnosis:
//! - A diagnosis is incomplete without a conclusion state
//! - If conclusion=uncertain, Anna must explicitly say uncertainty
//! - No confident language allowed when uncertain

mod diagnosis_builder;
mod diagnosis_types;
mod diagnosis_validation;

pub use diagnosis_builder::DiagnosisBuilder;
pub use diagnosis_types::{ConclusionState, DiagnosisConclusion};
pub use diagnosis_validation::{
    ConclusionLanguageValidator, ConclusionValidation, LanguageValidation,
};
