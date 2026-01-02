//! ERA Pipeline types - Stage definitions and intent types.

/// Pipeline stage identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    /// Extract intent and required facts.
    Evidence,
    /// Reason over evidence only.
    Reasoning,
    /// Convert reasoning to user answer.
    Answer,
}

impl PipelineStage {
    /// Get stage label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Evidence => "EVIDENCE",
            Self::Reasoning => "REASONING",
            Self::Answer => "ANSWER",
        }
    }

    /// Get next stage.
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Evidence => Some(Self::Reasoning),
            Self::Reasoning => Some(Self::Answer),
            Self::Answer => None,
        }
    }
}

/// Intent extracted from user question.
#[derive(Debug, Clone)]
pub struct ExtractedIntent {
    /// Case ID.
    pub case_id: String,
    /// Raw question from user.
    pub raw_question: String,
    /// Canonical intent (e.g., "memory.free", "boot.slow_service").
    pub intent: String,
    /// Required facts to answer this question.
    pub required_facts: Vec<String>,
    /// Expected answer type.
    pub answer_type: AnswerType,
}

/// Expected answer type for translator precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerType {
    /// Numeric value (e.g., "17.0 GiB").
    Numeric,
    /// Yes/No + 1 sentence max.
    Boolean,
    /// List of items.
    List,
    /// Single entity name.
    Entity,
    /// Brief explanation (2-3 sentences max).
    Brief,
}

impl AnswerType {
    /// Parse from intent pattern.
    pub fn from_intent(intent: &str) -> Self {
        let lower = intent.to_lowercase();

        // Numeric patterns
        if lower.contains("how much")
            || lower.contains("how many")
            || lower.contains("size")
            || lower.contains("free")
            || lower.contains("usage")
            || lower.contains("time")
            || lower.contains("temp")
        {
            return Self::Numeric;
        }

        // Boolean patterns
        if lower.contains("is_")
            || lower.contains("has_")
            || lower.contains("enabled")
            || lower.contains("running")
            || lower.contains("installed")
        {
            return Self::Boolean;
        }

        // List patterns
        if lower.contains("which")
            || lower.contains("list")
            || lower.contains("all_")
            || lower.contains("failed")
        {
            return Self::List;
        }

        // Entity patterns
        if lower.contains("what_")
            || lower.contains("model")
            || lower.contains("version")
            || lower.contains("name")
        {
            return Self::Entity;
        }

        Self::Brief
    }

    /// Get label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Boolean => "boolean",
            Self::List => "list",
            Self::Entity => "entity",
            Self::Brief => "brief",
        }
    }
}

/// Probe mapping for required facts.
#[derive(Debug, Clone)]
pub struct ProbeMapping {
    /// Fact name (e.g., "memory.free_gib").
    pub fact_name: String,
    /// Probe ID to run.
    pub probe_id: String,
    /// Extraction function name.
    pub extractor: String,
}
