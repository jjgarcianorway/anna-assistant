//! QuestionIntent v1 (Part A) - v0.0.437.
//!
//! The translator model must output exactly one QuestionIntent before
//! anything else happens. This is the typed contract for understanding.

pub mod builder;
pub mod constraints;
pub mod enums;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use builder::IntentBuilder;
pub use constraints::{AnswerConstraints, ClarificationRequest, Units};
pub use enums::{IntentCategory, Precision, Scope, Subject, Timeframe};
pub use types::QuestionIntent;
