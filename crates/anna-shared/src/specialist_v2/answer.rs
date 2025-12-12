//! Answer types for specialist responses (v0.0.421).
//!
//! Defines the structured answer types that replace freeform text blobs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Direct answer for factual questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectAnswer {
    /// One sentence, user-facing answer
    pub short_text: String,

    /// Optional machine-friendly metrics
    #[serde(default)]
    pub metrics: Option<HashMap<String, serde_json::Value>>,
}

impl DirectAnswer {
    /// Create a simple direct answer with just text
    pub fn simple(text: &str) -> Self {
        Self {
            short_text: text.to_string(),
            metrics: None,
        }
    }

    /// Create a direct answer with text and metrics
    pub fn with_metrics(text: &str, metrics: HashMap<String, serde_json::Value>) -> Self {
        Self {
            short_text: text.to_string(),
            metrics: Some(metrics),
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, key: &str, value: serde_json::Value) {
        self.metrics
            .get_or_insert_with(HashMap::new)
            .insert(key.to_string(), value);
    }

    /// Create a yes answer
    pub fn yes(detail: &str) -> Self {
        Self::simple(&format!("Yes, {}", detail))
    }

    /// Create a no answer
    pub fn no(detail: &str) -> Self {
        Self::simple(&format!("No, {}", detail))
    }
}

/// A key finding from the evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFinding {
    /// Label for the finding (e.g., "memory_available", "boot_time")
    pub label: String,

    /// Value of the finding (e.g., "17.0 GiB", "25.6s")
    pub value: String,

    /// Optional severity: info, warning, critical
    #[serde(default)]
    pub severity: Option<FindingSeverity>,

    /// Evidence sources (probe IDs)
    #[serde(default)]
    pub evidence: Vec<String>,
}

impl KeyFinding {
    /// Create a new finding
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            severity: None,
            evidence: vec![],
        }
    }

    /// Builder: set severity
    pub fn with_severity(mut self, severity: FindingSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Builder: add evidence source
    pub fn with_evidence(mut self, probe_id: &str) -> Self {
        self.evidence.push(probe_id.to_string());
        self
    }

    /// Create an info finding
    pub fn info(label: &str, value: &str) -> Self {
        Self::new(label, value).with_severity(FindingSeverity::Info)
    }

    /// Create a warning finding
    pub fn warning(label: &str, value: &str) -> Self {
        Self::new(label, value).with_severity(FindingSeverity::Warning)
    }

    /// Create a critical finding
    pub fn critical(label: &str, value: &str) -> Self {
        Self::new(label, value).with_severity(FindingSeverity::Critical)
    }
}

/// Severity of a finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    #[default]
    Info,
    Warning,
    Critical,
}

/// A recommended action for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    /// Action label (e.g., "cleanup_root_fs", "enable_trim")
    pub label: String,

    /// Human-readable summary (1-2 sentences)
    pub summary: String,

    /// Risk level: low, medium, high
    #[serde(default)]
    pub risk_level: RiskLevel,

    /// Whether this needs user confirmation before execution
    #[serde(default)]
    pub needs_confirmation: bool,
}

impl RecommendedAction {
    /// Create a new action
    pub fn new(label: &str, summary: &str) -> Self {
        Self {
            label: label.to_string(),
            summary: summary.to_string(),
            risk_level: RiskLevel::Low,
            needs_confirmation: false,
        }
    }

    /// Builder: set risk level
    pub fn with_risk(mut self, risk: RiskLevel) -> Self {
        self.risk_level = risk;
        self.needs_confirmation = matches!(risk, RiskLevel::Medium | RiskLevel::High);
        self
    }

    /// Create a low-risk action
    pub fn low_risk(label: &str, summary: &str) -> Self {
        Self::new(label, summary).with_risk(RiskLevel::Low)
    }

    /// Create a medium-risk action
    pub fn medium_risk(label: &str, summary: &str) -> Self {
        Self::new(label, summary).with_risk(RiskLevel::Medium)
    }

    /// Create a high-risk action
    pub fn high_risk(label: &str, summary: &str) -> Self {
        Self::new(label, summary).with_risk(RiskLevel::High)
    }
}

/// Risk level for an action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

/// Answer type for routing to correct response format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerType {
    /// Pure fact: "How much free RAM?"
    Fact,
    /// Yes/no: "Do I have failed services?"
    YesNo,
    /// Short "what is": "What GPU driver?"
    WhatIs,
    /// Diagnostic "why": "Why is boot slow?"
    Diagnostic,
    /// How-to (only when explicitly requested)
    HowTo,
    /// Unknown/general
    General,
}

impl AnswerType {
    /// Detect answer type from intent string
    pub fn from_intent(intent: &str) -> Self {
        let intent_lower = intent.to_lowercase();

        // Yes/no patterns
        if intent_lower.starts_with("check_")
            || intent_lower.starts_with("is_")
            || intent_lower.starts_with("has_")
            || intent_lower.starts_with("are_")
            || intent_lower.contains("_enabled")
            || intent_lower.contains("_active")
            || intent_lower.contains("_installed")
            || intent_lower.contains("_failed")
        {
            return Self::YesNo;
        }

        // Fact patterns
        if intent_lower.starts_with("show_")
            || intent_lower.starts_with("get_")
            || intent_lower.starts_with("list_")
            || intent_lower.contains("_usage")
            || intent_lower.contains("_status")
            || intent_lower.contains("_info")
        {
            return Self::Fact;
        }

        // What-is patterns
        if intent_lower.starts_with("what_") || intent_lower.starts_with("which_") {
            return Self::WhatIs;
        }

        // Diagnostic patterns
        if intent_lower.starts_with("why_")
            || intent_lower.starts_with("diagnose_")
            || intent_lower.contains("_slow")
            || intent_lower.contains("_problem")
            || intent_lower.contains("_issue")
        {
            return Self::Diagnostic;
        }

        // How-to patterns
        if intent_lower.starts_with("how_")
            || intent_lower.starts_with("enable_")
            || intent_lower.starts_with("configure_")
            || intent_lower.starts_with("setup_")
        {
            return Self::HowTo;
        }

        Self::General
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_answer() {
        let answer = DirectAnswer::simple("17.0 GiB available");
        assert_eq!(answer.short_text, "17.0 GiB available");
        assert!(answer.metrics.is_none());
    }

    #[test]
    fn test_yes_no_answers() {
        let yes = DirectAnswer::yes("there are 2 failed services.");
        assert!(yes.short_text.starts_with("Yes,"));

        let no = DirectAnswer::no("there are no failed services.");
        assert!(no.short_text.starts_with("No,"));
    }

    #[test]
    fn test_key_finding() {
        let finding = KeyFinding::critical("disk_usage", "95%").with_evidence("probe:df");

        assert_eq!(finding.label, "disk_usage");
        assert_eq!(finding.severity, Some(FindingSeverity::Critical));
        assert_eq!(finding.evidence, vec!["probe:df"]);
    }

    #[test]
    fn test_answer_type_detection() {
        assert_eq!(
            AnswerType::from_intent("check_failed_services"),
            AnswerType::YesNo
        );
        assert_eq!(
            AnswerType::from_intent("is_swap_enabled"),
            AnswerType::YesNo
        );
        assert_eq!(
            AnswerType::from_intent("show_memory_usage"),
            AnswerType::Fact
        );
        assert_eq!(
            AnswerType::from_intent("what_gpu_driver"),
            AnswerType::WhatIs
        );
        assert_eq!(
            AnswerType::from_intent("why_boot_slow"),
            AnswerType::Diagnostic
        );
        assert_eq!(
            AnswerType::from_intent("enable_vim_syntax"),
            AnswerType::HowTo
        );
    }
}
