//! Core loop type definitions.
//!
//! This module contains all data structures used by the core loop:
//! - CoreLoopResult: The final result returned to the user
//! - AnswerSource: Where the answer came from (recipe/specialist)
//! - InternalComm: Internal communication messages
//! - ParsedQuery: Structured query from translator
//! - SpecialistSolution: Solution from specialist

use std::collections::HashMap;

/// Result of the core loop
#[derive(Debug)]
pub struct CoreLoopResult {
    pub answer: String,
    pub source: AnswerSource,
    pub recipe_id: Option<String>,
    pub reliability: u8,
    pub elapsed_ms: u64,
    pub internal_comms: Vec<InternalComm>,
}

/// Where the answer came from
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerSource {
    /// Answered from a learned recipe (instant)
    Recipe,
    /// Answered by specialist (LLM), now learned
    Specialist { name: String, learned: bool },
    /// Failed to get an answer
    Failed,
}

/// Internal communication entry (fly-on-wall experience)
#[derive(Debug, Clone)]
pub struct InternalComm {
    pub from: String,
    pub message: String,
    pub elapsed_ms: u64,
}

/// Parsed query from translator
#[derive(Debug, Clone)]
pub struct ParsedQuery {
    pub intent: String,
    pub domain: String,
    pub probes: Vec<String>,
    pub entities: HashMap<String, String>,
}

/// Solution from specialist
#[derive(Debug)]
pub struct SpecialistSolution {
    pub answer: String,
    pub confidence: f32,
    pub explanation: String,
}
