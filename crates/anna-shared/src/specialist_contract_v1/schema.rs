//! Specialist Response Contract v1 (Part A) - v0.0.440.
//!
//! Every specialist MUST output ONLY this JSON, nothing else.
//! No markdown. No tables. No extra keys.
//!
//! Hard limits:
//! - summary: max 140 chars
//! - actions: max 5
//! - citations: max 5

use serde::{Deserialize, Serialize};

/// Maximum characters for summary.
pub const MAX_SUMMARY_CHARS: usize = 140;

/// Maximum actions per response.
pub const MAX_ACTIONS: usize = 5;

/// Maximum citations per response.
pub const MAX_CITATIONS: usize = 5;

/// Maximum characters for citation snippet.
pub const MAX_SNIPPET_CHARS: usize = 140;

/// Department that handled the case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SrcDepartment {
    Performance,
    Storage,
    Services,
    Network,
    Security,
    Hardware,
    Desktop,
}

impl SrcDepartment {
    /// Parse from string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "performance" | "perf" => Some(Self::Performance),
            "storage" | "disk" => Some(Self::Storage),
            "services" | "service" | "svc" => Some(Self::Services),
            "network" | "net" => Some(Self::Network),
            "security" | "sec" => Some(Self::Security),
            "hardware" | "hw" => Some(Self::Hardware),
            "desktop" | "de" => Some(Self::Desktop),
            _ => None,
        }
    }

    /// Label for serialization.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Performance => "Performance",
            Self::Storage => "Storage",
            Self::Services => "Services",
            Self::Network => "Network",
            Self::Security => "Security",
            Self::Hardware => "Hardware",
            Self::Desktop => "Desktop",
        }
    }
}

/// Risk level for proposed actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SrcRisk {
    /// Read-only operations (probes, status checks).
    ReadOnly,
    /// Safe changes (enabling a service, changing a setting).
    SafeChange,
    /// Risky changes (format, delete, kernel updates).
    RiskyChange,
}

impl SrcRisk {
    /// Label for serialization.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::SafeChange => "safe_change",
            Self::RiskyChange => "risky_change",
        }
    }
}

impl Default for SrcRisk {
    fn default() -> Self {
        Self::ReadOnly
    }
}

/// Assessment section of SRC v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrcAssessment {
    /// One sentence summary, no markdown, max 140 chars.
    pub summary: String,
    /// Confidence level 0.0-1.0.
    pub confidence: f64,
    /// Risk level.
    pub risk: SrcRisk,
}

impl SrcAssessment {
    /// Create a new assessment.
    pub fn new(summary: &str, confidence: f64, risk: SrcRisk) -> Self {
        Self {
            summary: truncate_str(summary, MAX_SUMMARY_CHARS),
            confidence: confidence.clamp(0.0, 1.0),
            risk,
        }
    }

    /// Validate the assessment.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.summary.is_empty() {
            errors.push("summary cannot be empty".to_string());
        }
        if self.summary.len() > MAX_SUMMARY_CHARS {
            errors.push(format!("summary exceeds {} chars", MAX_SUMMARY_CHARS));
        }
        if self.summary.contains('#') || self.summary.contains('*') || self.summary.contains('`') {
            errors.push("summary contains markdown (# * `)".to_string());
        }
        if self.confidence < 0.0 || self.confidence > 1.0 {
            errors.push("confidence must be 0.0-1.0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SrcActionType {
    /// Run a probe for more information.
    Probe,
    /// Explain something to the user.
    Explain,
    /// Make a change to the system.
    Change,
}

/// A proposed action in SRC v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrcAction {
    /// Action type.
    #[serde(rename = "type")]
    pub action_type: SrcActionType,
    /// Short title.
    pub title: String,
    /// Shell command to execute, or null.
    #[serde(default)]
    pub command: Option<String>,
    /// Why this action is needed.
    pub why: String,
    /// Expected outcome.
    pub expected: String,
    /// Rollback command if change fails, or null.
    #[serde(default)]
    pub rollback: Option<String>,
}

impl SrcAction {
    /// Create a probe action.
    pub fn probe(title: &str, command: &str, why: &str, expected: &str) -> Self {
        Self {
            action_type: SrcActionType::Probe,
            title: title.to_string(),
            command: Some(command.to_string()),
            why: why.to_string(),
            expected: expected.to_string(),
            rollback: None,
        }
    }

    /// Create an explain action.
    pub fn explain(title: &str, why: &str, expected: &str) -> Self {
        Self {
            action_type: SrcActionType::Explain,
            title: title.to_string(),
            command: None,
            why: why.to_string(),
            expected: expected.to_string(),
            rollback: None,
        }
    }

    /// Create a change action.
    pub fn change(title: &str, command: &str, why: &str, expected: &str, rollback: Option<&str>) -> Self {
        Self {
            action_type: SrcActionType::Change,
            title: title.to_string(),
            command: Some(command.to_string()),
            why: why.to_string(),
            expected: expected.to_string(),
            rollback: rollback.map(String::from),
        }
    }

    /// Validate the action.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.is_empty() {
            errors.push("action title cannot be empty".to_string());
        }
        if self.why.is_empty() {
            errors.push("action why cannot be empty".to_string());
        }
        if self.expected.is_empty() {
            errors.push("action expected cannot be empty".to_string());
        }

        // Probe and Change need commands
        if matches!(self.action_type, SrcActionType::Probe | SrcActionType::Change) {
            if self.command.is_none() {
                errors.push(format!("{:?} action needs a command", self.action_type));
            }
        }

        // Risky changes should have rollback
        if self.action_type == SrcActionType::Change && self.rollback.is_none() {
            // Warning, not error - some changes can't be rolled back
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Citation source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SrcCitationSource {
    /// Man page.
    Man,
    /// Arch Wiki.
    ArchWiki,
    /// --help output.
    Help,
    /// Local documentation.
    LocalDoc,
}

/// A citation in SRC v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrcCitation {
    /// Source type.
    pub source: SrcCitationSource,
    /// Reference identifier (e.g., "systemd-analyze(1)" or "ArchWiki:Systemd").
    #[serde(rename = "ref")]
    pub reference: String,
    /// Relevant snippet, max 140 chars.
    pub snippet: String,
}

impl SrcCitation {
    /// Create a new citation.
    pub fn new(source: SrcCitationSource, reference: &str, snippet: &str) -> Self {
        Self {
            source,
            reference: reference.to_string(),
            snippet: truncate_str(snippet, MAX_SNIPPET_CHARS),
        }
    }

    /// Validate the citation.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.reference.is_empty() {
            errors.push("citation ref cannot be empty".to_string());
        }
        if self.snippet.len() > MAX_SNIPPET_CHARS {
            errors.push(format!("citation snippet exceeds {} chars", MAX_SNIPPET_CHARS));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// The Specialist Response Contract v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResponseV1 {
    /// Case ID (must match the ticket).
    pub case_id: String,
    /// Department that handled this.
    pub department: SrcDepartment,
    /// Assessment of the situation.
    pub assessment: SrcAssessment,
    /// Proposed actions (max 5).
    #[serde(default)]
    pub actions: Vec<SrcAction>,
    /// Citations (max 5).
    #[serde(default)]
    pub citations: Vec<SrcCitation>,
}

impl SpecialistResponseV1 {
    /// Create a new response.
    pub fn new(
        case_id: &str,
        department: SrcDepartment,
        summary: &str,
        confidence: f64,
        risk: SrcRisk,
    ) -> Self {
        Self {
            case_id: case_id.to_string(),
            department,
            assessment: SrcAssessment::new(summary, confidence, risk),
            actions: Vec::new(),
            citations: Vec::new(),
        }
    }

    /// Add an action.
    pub fn with_action(mut self, action: SrcAction) -> Self {
        if self.actions.len() < MAX_ACTIONS {
            self.actions.push(action);
        }
        self
    }

    /// Add a citation.
    pub fn with_citation(mut self, citation: SrcCitation) -> Self {
        if self.citations.len() < MAX_CITATIONS {
            self.citations.push(citation);
        }
        self
    }

    /// Validate the entire response.
    pub fn validate(&self, expected_case_id: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Case ID must match
        if self.case_id != expected_case_id {
            errors.push(format!(
                "case_id mismatch: expected '{}', got '{}'",
                expected_case_id, self.case_id
            ));
        }

        // Validate assessment
        if let Err(assessment_errors) = self.assessment.validate() {
            errors.extend(assessment_errors);
        }

        // Validate actions count
        if self.actions.len() > MAX_ACTIONS {
            errors.push(format!("too many actions: {} > {}", self.actions.len(), MAX_ACTIONS));
        }

        // Validate each action
        for (i, action) in self.actions.iter().enumerate() {
            if let Err(action_errors) = action.validate() {
                for e in action_errors {
                    errors.push(format!("action[{}]: {}", i, e));
                }
            }
        }

        // Validate citations count
        if self.citations.len() > MAX_CITATIONS {
            errors.push(format!("too many citations: {} > {}", self.citations.len(), MAX_CITATIONS));
        }

        // Validate each citation
        for (i, citation) in self.citations.iter().enumerate() {
            if let Err(citation_errors) = citation.validate() {
                for e in citation_errors {
                    errors.push(format!("citation[{}]: {}", i, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Serialize to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Truncate string to max length.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_src_department_parse() {
        assert_eq!(SrcDepartment::from_str_loose("Performance"), Some(SrcDepartment::Performance));
        assert_eq!(SrcDepartment::from_str_loose("hardware"), Some(SrcDepartment::Hardware));
        assert_eq!(SrcDepartment::from_str_loose("bogus"), None);
    }

    #[test]
    fn test_src_assessment_validation() {
        let valid = SrcAssessment::new("Boot time is 7.5 seconds.", 0.9, SrcRisk::ReadOnly);
        assert!(valid.validate().is_ok());

        let with_markdown = SrcAssessment {
            summary: "# Boot time is slow".to_string(),
            confidence: 0.9,
            risk: SrcRisk::ReadOnly,
        };
        assert!(with_markdown.validate().is_err());
    }

    #[test]
    fn test_src_action_probe() {
        let action = SrcAction::probe(
            "Check boot time",
            "systemd-analyze",
            "Need boot breakdown",
            "Time breakdown",
        );
        assert!(action.validate().is_ok());
        assert_eq!(action.action_type, SrcActionType::Probe);
    }

    #[test]
    fn test_src_response_validation() {
        let response = SpecialistResponseV1::new(
            "DSK-0101",
            SrcDepartment::Performance,
            "Boot time is 7.5 seconds.",
            0.9,
            SrcRisk::ReadOnly,
        );

        assert!(response.validate("DSK-0101").is_ok());
        assert!(response.validate("DSK-0102").is_err()); // Wrong case_id
    }

    #[test]
    fn test_src_response_max_actions() {
        let mut response = SpecialistResponseV1::new(
            "DSK-0101",
            SrcDepartment::Performance,
            "Summary",
            0.9,
            SrcRisk::ReadOnly,
        );

        // Add 10 actions - should only keep 5
        for i in 0..10 {
            response = response.with_action(SrcAction::explain(
                &format!("Action {}", i),
                "why",
                "expected",
            ));
        }

        assert_eq!(response.actions.len(), MAX_ACTIONS);
    }

    #[test]
    fn test_truncate_str() {
        let short = "hello";
        assert_eq!(truncate_str(short, 10), "hello");

        let long = "a".repeat(200);
        let truncated = truncate_str(&long, 50);
        assert!(truncated.len() <= 50);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_json_serialization() {
        let response = SpecialistResponseV1::new(
            "DSK-0101",
            SrcDepartment::Performance,
            "Boot time is 7.5 seconds.",
            0.9,
            SrcRisk::ReadOnly,
        )
        .with_citation(SrcCitation::new(
            SrcCitationSource::Man,
            "systemd-analyze(1)",
            "Analyze and debug system manager",
        ));

        let json = response.to_json().unwrap();
        assert!(json.contains("DSK-0101"));
        assert!(json.contains("Performance"));
    }
}
