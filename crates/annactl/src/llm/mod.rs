//! LLM Module - Organized LLM functionality
//!
//! Beta.200: Clean separation of LLM concerns
//!
//! This module organizes all LLM-related functionality into focused sub-modules:
//! - intent: Query intent detection and classification
//! - recipe_matcher: Recipe matching based on intent
//! - answer_generator: Natural language answer generation

pub mod answer_generator;
pub mod intent;
pub mod recipe_matcher;

// Re-export key types for convenience
pub use answer_generator::AnswerGenerator;
pub use intent::{detect_intent, Intent};
pub use recipe_matcher::RecipeMatcher;
