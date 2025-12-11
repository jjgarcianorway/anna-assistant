//! Specialist JSON contract (v0.0.404).
//!
//! This defines the STRICT schema that all specialists must output.
//! Specialists ONLY output JSON - no prose, no roleplay, no excuses.
//! The personality layer (Sofia, Tomas, etc.) is handled by the renderer.
//!
//! Key principles:
//! - answer.short MUST directly answer the user's question
//! - evidence[] MUST back up every claim
//! - discovery.new_probes/recipes is how Anna learns new capabilities
//! - Specialists NEVER speak to the user, only return structured data

use serde::{Deserialize, Serialize};

/// Input to a specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistInput {
    pub ticket_id: String,
    pub domain: String,
    pub intent: SpecialistIntent,
    pub question: String,
    pub probes: std::collections::HashMap<String, String>,
}

/// What kind of request this is
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistIntent {
    Question,
    Investigate,
    Request,
}

/// The complete response from a specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResponse {
    pub ticket_id: String,
    pub status: ResponseStatus,
    pub answer: Answer,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub staff_view: Option<StaffView>,
    #[serde(default)]
    pub next_steps: Option<NextSteps>,
    #[serde(default)]
    pub discovery: Option<Discovery>,
    /// If status is needs_more_data, which probes are missing
    #[serde(default)]
    pub missing_probes: Vec<String>,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ok,
    NeedsMoreData,
    CannotAnswer,
    Error,
}

/// The actual answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// Direct one-sentence answer to the question
    pub short: String,
    /// Optional longer explanation
    #[serde(default)]
    pub detail: Option<String>,
    /// Domain-specific structured data
    #[serde(default)]
    pub domain_summary: Option<serde_json::Value>,
}

/// Evidence backing up a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Which probe this evidence comes from
    pub probe: String,
    /// Short relevant excerpt from the probe output
    pub snippet: String,
    /// What this snippet means for the answer
    pub interpretation: String,
}

/// Internal staff view (for personality rendering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffView {
    #[serde(default = "default_role")]
    pub assignee_role: String,
    #[serde(default)]
    pub severity: Severity,
    #[serde(default)]
    pub mood: Mood,
    /// Short internal note (NOT for user)
    #[serde(default)]
    pub short_note: Option<String>,
    #[serde(default = "default_complexity")]
    pub complexity: u8,
}

fn default_role() -> String {
    "System Specialist".to_string()
}

fn default_complexity() -> u8 {
    1
}

/// Severity of the finding
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Critical,
    Unknown,
}

/// Specialist's confidence/mood
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Mood {
    #[default]
    Confident,
    Uncertain,
    Blocked,
}

/// Next steps (user actions and internal actions)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NextSteps {
    #[serde(default)]
    pub user_actions: Vec<UserAction>,
    #[serde(default)]
    pub internal_actions: Vec<InternalAction>,
}

/// Action the user can take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAction {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub recipe_id: Option<String>,
}

/// Internal action (run more probes, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAction {
    pub id: String,
    #[serde(default)]
    pub probes: Vec<String>,
}

/// Discovery: how specialists propose new probes and recipes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Discovery {
    #[serde(default)]
    pub new_probes: Vec<ProbeProposal>,
    #[serde(default)]
    pub new_recipes: Vec<RecipeProposal>,
}

/// A proposed new probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeProposal {
    pub id: String,
    pub intent: String,
    pub domain: String,
    pub command: String,
    #[serde(default)]
    pub parse_hint: Option<String>,
    #[serde(default)]
    pub reusable_for: Vec<String>,
}

/// A proposed new recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeProposal {
    pub id: String,
    pub intent: String,
    pub domain: String,
    pub summary: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default = "default_risk")]
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub steps_high_level: Vec<String>,
    #[serde(default)]
    pub reusable_for: Vec<String>,
}

fn default_risk() -> RiskLevel {
    RiskLevel::Low
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

impl SpecialistResponse {
    /// Create a fallback response when JSON parsing fails
    pub fn parse_error(ticket_id: &str, error: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Error,
            answer: Answer {
                short: "Failed to parse specialist response.".to_string(),
                detail: Some(format!("Parse error: {}", error)),
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.0,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Unknown,
                mood: Mood::Blocked,
                short_note: Some("JSON parse failed".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
        }
    }

    /// Create a response from deterministic/direct answer
    pub fn from_direct_answer(ticket_id: &str, answer: &str, evidence_probe: &str, evidence_snippet: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: answer.to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: evidence_probe.to_string(),
                snippet: evidence_snippet.to_string(),
                interpretation: "Directly extracted from probe output.".to_string(),
            }],
            confidence: 0.95,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Info,
                mood: Mood::Confident,
                short_note: Some("Direct answer from probe data.".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_specialist_response() {
        let json = r#"{
            "ticket_id": "DSK-0101",
            "status": "ok",
            "answer": {
                "short": "No, there is no active swap configured.",
                "detail": "Both free -h and /proc/swaps show 0B swap."
            },
            "evidence": [
                {
                    "probe": "swap_files",
                    "snippet": "Filename Type Size Used Priority",
                    "interpretation": "No entries listed."
                }
            ],
            "confidence": 0.95,
            "staff_view": {
                "assignee_role": "System Specialist",
                "severity": "info",
                "mood": "confident",
                "short_note": "No swap configured.",
                "complexity": 1
            }
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::Ok);
        assert!(response.answer.short.contains("swap"));
        assert_eq!(response.evidence.len(), 1);
    }

    #[test]
    fn test_parse_needs_more_data() {
        let json = r#"{
            "ticket_id": "DSK-0102",
            "status": "needs_more_data",
            "answer": {
                "short": "I cannot determine if zram is enabled."
            },
            "missing_probes": ["zram_devices"],
            "confidence": 0.3
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::NeedsMoreData);
        assert_eq!(response.missing_probes, vec!["zram_devices"]);
    }

    #[test]
    fn test_parse_with_discovery() {
        let json = r#"{
            "ticket_id": "DSK-0103",
            "status": "ok",
            "answer": {
                "short": "Test answer"
            },
            "evidence": [],
            "confidence": 0.8,
            "discovery": {
                "new_probes": [
                    {
                        "id": "zram_devices",
                        "intent": "Detect zram configuration",
                        "domain": "system",
                        "command": "lsblk | grep zram",
                        "reusable_for": ["is zram enabled", "compressed memory"]
                    }
                ],
                "new_recipes": []
            }
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert!(response.discovery.is_some());
        let discovery = response.discovery.unwrap();
        assert_eq!(discovery.new_probes.len(), 1);
        assert_eq!(discovery.new_probes[0].id, "zram_devices");
    }
}
