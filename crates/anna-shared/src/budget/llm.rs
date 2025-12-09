//! LLM budget management (v0.0.199).

use serde::{Deserialize, Serialize};

use super::types::{
    Stage, LLM_MAX_CONTEXT_TOKENS, LLM_MAX_DRAFT_TOKENS, LLM_MAX_SPECIALIST_TOKENS,
    SPECIALIST_TIMEOUT_SECS, TRANSLATOR_TIMEOUT_SECS,
};

/// Configurable LLM token budgets for local inference (v0.0.41)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LlmBudget {
    /// Max tokens for draft/translation responses
    pub max_draft_tokens: u32,
    /// Max tokens for specialist responses
    pub max_specialist_tokens: u32,
    /// Max context tokens (prompt + response)
    pub max_context_tokens: u32,
    /// Translator timeout in seconds
    pub translator_timeout_secs: u64,
    /// Specialist timeout in seconds
    pub specialist_timeout_secs: u64,
}

impl Default for LlmBudget {
    fn default() -> Self {
        Self {
            max_draft_tokens: LLM_MAX_DRAFT_TOKENS,
            max_specialist_tokens: LLM_MAX_SPECIALIST_TOKENS,
            max_context_tokens: LLM_MAX_CONTEXT_TOKENS,
            translator_timeout_secs: TRANSLATOR_TIMEOUT_SECS,
            specialist_timeout_secs: SPECIALIST_TIMEOUT_SECS,
        }
    }
}

impl LlmBudget {
    /// Create tight budget for fast path (minimal tokens)
    pub fn fast_path() -> Self {
        Self {
            max_draft_tokens: 400,
            max_specialist_tokens: 600,
            max_context_tokens: 4000,
            translator_timeout_secs: 15,
            specialist_timeout_secs: 20,
        }
    }

    /// Create standard budget for normal queries
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create extended budget for complex queries
    pub fn extended() -> Self {
        Self {
            max_draft_tokens: 1200,
            max_specialist_tokens: 2000,
            max_context_tokens: 8000,
            translator_timeout_secs: 60,
            specialist_timeout_secs: 90,
        }
    }

    /// Check if elapsed time exceeds translator timeout
    pub fn is_translator_timeout(&self, elapsed_secs: u64) -> bool {
        elapsed_secs > self.translator_timeout_secs
    }

    /// Check if elapsed time exceeds specialist timeout
    pub fn is_specialist_timeout(&self, elapsed_secs: u64) -> bool {
        elapsed_secs > self.specialist_timeout_secs
    }
}

/// Result of LLM fallback decision (v0.0.41)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmFallback {
    /// Continue normally
    Continue,
    /// Translator timed out, use fallback interpretation
    TranslatorTimeout { elapsed_secs: u64 },
    /// Specialist timed out, return raw probe data
    SpecialistTimeout { elapsed_secs: u64 },
}

impl LlmFallback {
    /// Check if any timeout occurred
    pub fn is_timeout(&self) -> bool {
        !matches!(self, Self::Continue)
    }

    /// Get fallback message for user display
    pub fn fallback_message(&self) -> Option<String> {
        match self {
            Self::Continue => None,
            Self::TranslatorTimeout { elapsed_secs } => Some(format!(
                "Request interpretation took too long ({}s). Using simplified routing.",
                elapsed_secs
            )),
            Self::SpecialistTimeout { elapsed_secs } => Some(format!(
                "Analysis took too long ({}s). Here's the raw system data:",
                elapsed_secs
            )),
        }
    }
}

/// Check if should fall back based on elapsed time (v0.0.41)
pub fn check_llm_fallback(stage: Stage, elapsed_secs: u64, budget: &LlmBudget) -> LlmFallback {
    match stage {
        Stage::Translator => {
            if budget.is_translator_timeout(elapsed_secs) {
                LlmFallback::TranslatorTimeout { elapsed_secs }
            } else {
                LlmFallback::Continue
            }
        }
        Stage::Specialist => {
            if budget.is_specialist_timeout(elapsed_secs) {
                LlmFallback::SpecialistTimeout { elapsed_secs }
            } else {
                LlmFallback::Continue
            }
        }
        _ => LlmFallback::Continue,
    }
}
