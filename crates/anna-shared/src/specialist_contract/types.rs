//! Core types for specialist input/output.

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

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ok,
    NeedsMoreData,
    CannotAnswer,
    Error,
    /// v0.0.408: Probes ran but no relevant documentation found
    NoEvidence,
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

/// Next steps (user actions and internal actions)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NextSteps {
    #[serde(default)]
    pub user_actions: Vec<UserAction>,
    #[serde(default)]
    pub internal_actions: Vec<InternalAction>,
}
