//! Core types and enums for specialist response schema.

use serde::{Deserialize, Serialize};

/// Default confidence value for serde.
pub(crate) fn default_confidence() -> f32 {
    0.5
}

/// Complete specialist response - the ONLY allowed format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResponse {
    /// Ticket this response is for
    pub ticket_id: String,

    /// Specialist identity
    #[serde(default)]
    pub specialist: SpecialistIdentity,

    /// Response status
    #[serde(default)]
    pub status: ResponseStatus,

    /// One-line technical summary
    pub summary: String,

    /// Confidence score (0.0 - 1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f32,

    /// Severity level
    #[serde(default)]
    pub severity: Severity,

    /// Structured findings from probes/knowledge
    #[serde(default)]
    pub findings: Vec<Finding>,

    /// Short reasoning bullets (1-4 items)
    #[serde(default)]
    pub analysis: Vec<String>,

    /// Recommendations for the user
    #[serde(default)]
    pub recommendations: Vec<Recommendation>,

    /// Suggested shell actions
    #[serde(default)]
    pub actions: Vec<Action>,

    /// Knowledge sources used
    #[serde(default)]
    pub knowledge_citations: Vec<KnowledgeCitation>,

    /// Probes that were used
    #[serde(default)]
    pub probes_used: Vec<ProbeUsed>,

    /// Error information (if status is error)
    #[serde(default)]
    pub error: ErrorInfo,
}

/// Specialist identity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecialistIdentity {
    pub name: String,
    pub role: String,
    pub department: String,
    #[serde(default)]
    pub seniority: Seniority,
}

/// Seniority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Seniority {
    #[default]
    Junior,
    Senior,
}

/// Response status - strict semantics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Complete answer based on probes and/or knowledge
    #[default]
    Success,
    /// Some findings but important data missing or inconclusive
    Partial,
    /// Probes returned nothing useful for this question
    NoData,
    /// Question not in this specialist's domain
    Unsupported,
    /// Something went wrong (timeout, parse error, internal)
    Error,
}

impl ResponseStatus {
    /// Check if this is a successful status
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::Partial)
    }

    /// Check if specialist should route to another
    pub fn should_reroute(&self) -> bool {
        matches!(self, Self::Unsupported)
    }

    /// Check if this is an error status
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Success => "Complete answer available",
            Self::Partial => "Partial answer with some uncertainty",
            Self::NoData => "No data available to answer this question",
            Self::Unsupported => "Question outside specialist's domain",
            Self::Error => "An error occurred during processing",
        }
    }
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Critical,
}

impl Severity {
    /// Get emoji for display
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Info => "",
            Self::Warning => "⚠️",
            Self::Critical => "🔴",
        }
    }
}

/// A structured finding from evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding key (e.g., "mem_available_mb")
    pub key: String,
    /// Finding value (e.g., "17024")
    pub value: String,
    /// References to evidence (e.g., ["probe:free"])
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

impl Finding {
    /// Create a new finding
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            evidence_refs: vec![],
        }
    }

    /// Add evidence reference
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence_refs.push(evidence.to_string());
        self
    }
}

/// A recommendation for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Unique ID
    pub id: String,
    /// Short title
    pub title: String,
    /// Description
    pub description: String,
    /// Risk level
    #[serde(default)]
    pub risk_level: RiskLevel,
    /// Optional concrete actions
    #[serde(default)]
    pub actions: Vec<String>,
}

/// Risk level for recommendations and actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

/// A suggested shell action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Unique ID
    pub id: String,
    /// Short title
    pub title: String,
    /// Shell command
    pub command: String,
    /// Run as user or root
    #[serde(default)]
    pub run_as: RunAs,
    /// Risk level
    #[serde(default)]
    pub risk_level: RiskLevel,
    /// Whether to auto-run (engine decides)
    #[serde(default)]
    pub auto_run: bool,
}

/// Run as user or root
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RunAs {
    #[default]
    User,
    Root,
}

/// Knowledge citation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCitation {
    /// Citation ID (e.g., "wiki:systemd")
    pub id: String,
    /// Source name
    pub source: String,
    /// Topic
    pub topic: String,
    /// Relevance level
    #[serde(default)]
    pub relevance: Relevance,
}

/// Relevance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Relevance {
    Low,
    #[default]
    Medium,
    High,
}

/// Probe usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeUsed {
    /// Probe ID (e.g., "probe:free")
    pub id: String,
    /// Probe status
    #[serde(default)]
    pub status: ProbeStatus,
    /// Description
    pub description: String,
    /// Raw probe key
    #[serde(default)]
    pub raw_key: Option<String>,
}

/// Probe status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProbeStatus {
    #[default]
    Ok,
    Empty,
    Failed,
    Timeout,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorInfo {
    /// User-facing error message
    #[serde(default)]
    pub message: Option<String>,
    /// Error kind
    #[serde(default)]
    pub kind: Option<ErrorKind>,
    /// Internal details (may be hidden from user)
    #[serde(default)]
    pub details: Option<String>,
}

/// Error kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Timeout,
    ParseError,
    NoEvidence,
    Internal,
}
