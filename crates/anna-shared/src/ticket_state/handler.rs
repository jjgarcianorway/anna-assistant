//! Handler type for ticket processing

use serde::{Deserialize, Serialize};

/// Handler type for ticket processing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerType {
    /// Handled by a recipe
    Recipe { name: String },
    /// Handled by deterministic logic
    Deterministic { route: String },
    /// Handled by LLM solver
    LlmSolver { tier: SolverTier, model: String },
}

impl std::fmt::Display for HandlerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recipe { name } => write!(f, "recipe:{}", name),
            Self::Deterministic { route } => write!(f, "deterministic:{}", route),
            Self::LlmSolver { tier, model } => write!(f, "llm:{}:{}", tier, model),
        }
    }
}

/// LLM solver tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverTier {
    Junior,
    Senior,
}

impl std::fmt::Display for SolverTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
        }
    }
}
