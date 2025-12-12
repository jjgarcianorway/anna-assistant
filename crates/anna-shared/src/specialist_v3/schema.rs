//! Strict specialist response schema (v0.0.425).
//!
//! This is THE SINGLE CANONICAL SCHEMA for all specialist responses.
//! No freeform prose - JSON only.

use serde::{Deserialize, Serialize};

/// Default confidence value for serde.
fn default_confidence() -> f32 {
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

impl Default for SpecialistResponse {
    fn default() -> Self {
        Self {
            ticket_id: String::new(),
            specialist: SpecialistIdentity::default(),
            status: ResponseStatus::default(),
            summary: String::new(),
            confidence: 0.5,
            severity: Severity::default(),
            findings: vec![],
            analysis: vec![],
            recommendations: vec![],
            actions: vec![],
            knowledge_citations: vec![],
            probes_used: vec![],
            error: ErrorInfo::default(),
        }
    }
}

impl SpecialistResponse {
    /// Create a success response
    pub fn success(ticket_id: &str, summary: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Success,
            summary: summary.to_string(),
            confidence: 0.8,
            ..Default::default()
        }
    }

    /// Create a partial response
    pub fn partial(ticket_id: &str, summary: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Partial,
            summary: summary.to_string(),
            confidence: 0.5,
            ..Default::default()
        }
    }

    /// Create a no-data response
    pub fn no_data(ticket_id: &str, reason: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::NoData,
            summary: reason.to_string(),
            confidence: 0.1,
            ..Default::default()
        }
    }

    /// Create an error response
    pub fn error(ticket_id: &str, kind: ErrorKind, message: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Error,
            summary: "An error occurred".to_string(),
            confidence: 0.0,
            error: ErrorInfo {
                message: Some(message.to_string()),
                kind: Some(kind),
                details: None,
            },
            ..Default::default()
        }
    }

    /// Builder: set specialist identity
    pub fn with_specialist(mut self, name: &str, role: &str, department: &str) -> Self {
        self.specialist = SpecialistIdentity {
            name: name.to_string(),
            role: role.to_string(),
            department: department.to_string(),
            seniority: Seniority::Junior,
        };
        self
    }

    /// Builder: set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Builder: set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Builder: add finding
    pub fn with_finding(mut self, finding: Finding) -> Self {
        self.findings.push(finding);
        self
    }

    /// Builder: add analysis bullet
    pub fn with_analysis(mut self, bullet: &str) -> Self {
        self.analysis.push(bullet.to_string());
        self
    }

    /// Builder: add recommendation
    pub fn with_recommendation(mut self, rec: Recommendation) -> Self {
        self.recommendations.push(rec);
        self
    }

    /// Builder: add action
    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Builder: add probe used
    pub fn with_probe(mut self, probe: ProbeUsed) -> Self {
        self.probes_used.push(probe);
        self
    }

    /// Builder: add knowledge citation
    pub fn with_citation(mut self, citation: KnowledgeCitation) -> Self {
        self.knowledge_citations.push(citation);
        self
    }

    /// Check if this response is usable
    pub fn is_usable(&self) -> bool {
        self.status.is_success() && !self.summary.is_empty()
    }

    /// Get all citation IDs
    pub fn citation_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self
            .knowledge_citations
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        ids.extend(self.probes_used.iter().map(|p| p.id.as_str()));
        ids
    }

    /// Validate the response structure
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = vec![];

        if self.ticket_id.is_empty() {
            errors.push("ticket_id is required".to_string());
        }

        if self.summary.is_empty() && !self.status.is_error() {
            errors.push("summary is required for non-error responses".to_string());
        }

        if self.confidence < 0.0 || self.confidence > 1.0 {
            errors.push("confidence must be between 0.0 and 1.0".to_string());
        }

        if self.status == ResponseStatus::Error && self.error.message.is_none() {
            errors.push("error.message is required when status is error".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let resp = SpecialistResponse::success("DSK-001", "Memory is healthy")
            .with_specialist("Sofia", "System Admin", "desktop")
            .with_confidence(0.95)
            .with_finding(Finding::new("mem_available_mb", "17024").with_evidence("probe:free"));

        assert_eq!(resp.status, ResponseStatus::Success);
        assert!(resp.is_usable());
        assert!(resp.validate().is_ok());
    }

    #[test]
    fn test_error_response() {
        let resp = SpecialistResponse::error("DSK-002", ErrorKind::Timeout, "Request timed out");

        assert_eq!(resp.status, ResponseStatus::Error);
        assert!(!resp.is_usable());
        assert!(resp.validate().is_ok());
    }

    #[test]
    fn test_validation_failures() {
        let resp = SpecialistResponse {
            ticket_id: String::new(), // Missing
            status: ResponseStatus::Success,
            summary: String::new(), // Missing
            confidence: 1.5,        // Out of range
            ..Default::default()
        };

        let errors = resp.validate().unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_status_semantics() {
        assert!(ResponseStatus::Success.is_success());
        assert!(ResponseStatus::Partial.is_success());
        assert!(!ResponseStatus::NoData.is_success());
        assert!(ResponseStatus::Unsupported.should_reroute());
        assert!(ResponseStatus::Error.is_error());
    }

    #[test]
    fn test_json_roundtrip() {
        let resp = SpecialistResponse::success("DSK-003", "All clear")
            .with_finding(Finding::new("uptime", "3 days"));

        let json = serde_json::to_string(&resp).unwrap();
        let parsed: SpecialistResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.ticket_id, "DSK-003");
        assert_eq!(parsed.findings.len(), 1);
    }
}
