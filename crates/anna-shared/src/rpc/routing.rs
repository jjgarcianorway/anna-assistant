//! Specialist routing types (v0.0.220).

use serde::{Deserialize, Serialize};

/// Specialist domain for service desk routing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistDomain {
    #[default]
    System,
    Network,
    Storage,
    Security,
    Packages,
}

impl std::fmt::Display for SpecialistDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Network => write!(f, "network"),
            Self::Storage => write!(f, "storage"),
            Self::Security => write!(f, "security"),
            Self::Packages => write!(f, "packages"),
        }
    }
}

/// Intent classification from translator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueryIntent {
    #[default]
    Question,
    Request,
    Investigate,
}

impl std::fmt::Display for QueryIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Question => write!(f, "question"),
            Self::Request => write!(f, "request"),
            Self::Investigate => write!(f, "investigate"),
        }
    }
}

/// Translator ticket - structured output from LLM translator
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranslatorTicket {
    /// Query intent classification
    #[serde(default)]
    pub intent: QueryIntent,
    /// Target specialist domain
    #[serde(default)]
    pub domain: SpecialistDomain,
    /// Extracted entities (processes, services, mounts, etc.)
    #[serde(default)]
    pub entities: Vec<String>,
    /// Probe IDs needed from allowlist
    #[serde(default)]
    pub needs_probes: Vec<String>,
    /// Clarification question if query is ambiguous
    #[serde(default)]
    pub clarification_question: Option<String>,
    /// Translator confidence 0.0-1.0
    #[serde(default)]
    pub confidence: f32,
    /// v0.0.74: Answer contract defining what the answer should contain
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_contract: Option<crate::answer_contract::AnswerContract>,
}
