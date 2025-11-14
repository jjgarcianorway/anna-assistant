//! Stable JSON output types for annactl daily and status
//! Phase 4.9: User Control and JSON Output
//!
//! These types provide machine-readable output for scripts and monitoring tools.

use anna_common::caretaker_brain::{CaretakerIssue, IssueSeverity, IssueVisibility};
use anna_common::profile::MachineProfile;
use serde::Serialize;

/// Health summary for JSON output
#[derive(Debug, Clone, Serialize)]
pub struct HealthSummaryJson {
    pub ok: usize,
    pub warnings: usize,
    pub failures: usize,
}

/// Disk summary for JSON output
#[derive(Debug, Clone, Serialize)]
pub struct DiskSummaryJson {
    pub used_percent: f64,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

/// User decision info for JSON output
#[derive(Debug, Clone, Serialize)]
pub struct IssueDecisionJson {
    /// Decision kind: "none", "acknowledged", or "snoozed"
    pub kind: String,
    /// Snooze date in ISO 8601 format if snoozed
    pub snoozed_until: Option<String>,
}

/// Issue representation for JSON output
#[derive(Debug, Clone, Serialize)]
pub struct IssueJson {
    /// Stable issue key for tracking
    pub key: String,
    /// Human-readable title
    pub title: String,
    /// Severity: "critical", "warning", or "info"
    pub severity: String,
    /// Visibility: "normal", "low_priority", or "deemphasized"
    pub visibility: String,
    /// Category identifier (e.g. "disk_space", "pacman_locks")
    pub category: String,
    /// Brief explanation of the issue
    pub summary: String,
    /// Recommended action to take
    pub recommended_action: Option<String>,
    /// Repair action ID if repairable
    pub repair_action_id: Option<String>,
    /// Reference URL (usually Arch Wiki)
    pub reference: Option<String>,
    /// Estimated impact of fixing
    pub impact: Option<String>,
    /// User decision info
    pub decision: IssueDecisionJson,
}

/// JSON output for daily command
#[derive(Debug, Clone, Serialize)]
pub struct DailyJson {
    /// JSON schema version for stable scripting (Phase 5.1)
    pub schema_version: String,
    /// Machine profile: "Laptop", "Desktop", "Server-Like", or "Unknown"
    pub profile: String,
    /// Timestamp in ISO 8601 format
    pub timestamp: String,
    /// Health summary
    pub health: HealthSummaryJson,
    /// Disk summary (if available)
    pub disk: Option<DiskSummaryJson>,
    /// Visible issues (excluding deemphasized)
    pub issues: Vec<IssueJson>,
    /// Count of deemphasized issues (hidden from daily)
    pub deemphasized_issue_count: u32,
}

/// JSON output for status command
#[derive(Debug, Clone, Serialize)]
pub struct StatusJson {
    /// JSON schema version for stable scripting (Phase 5.1)
    pub schema_version: String,
    /// Machine profile: "Laptop", "Desktop", "Server-Like", or "Unknown"
    pub profile: String,
    /// Timestamp in ISO 8601 format
    pub timestamp: String,
    /// Health summary
    pub health: HealthSummaryJson,
    /// Disk summary (if available)
    pub disk: Option<DiskSummaryJson>,
    /// All issues including deemphasized
    pub issues: Vec<IssueJson>,
}

impl IssueJson {
    /// Convert CaretakerIssue to IssueJson
    pub fn from_caretaker_issue(issue: &CaretakerIssue) -> Self {
        let severity = match issue.severity {
            IssueSeverity::Critical => "critical",
            IssueSeverity::Warning => "warning",
            IssueSeverity::Info => "info",
        };

        let visibility = match issue.visibility {
            IssueVisibility::VisibleNormal => "normal",
            IssueVisibility::VisibleButLowPriority => "low_priority",
            IssueVisibility::Deemphasized => "deemphasized",
        };

        // Extract decision info
        let decision = if let Some((decision_type, snooze_date)) = &issue.decision_info {
            IssueDecisionJson {
                kind: decision_type.clone(),
                snoozed_until: snooze_date.clone(),
            }
        } else {
            IssueDecisionJson {
                kind: "none".to_string(),
                snoozed_until: None,
            }
        };

        // Generate category from repair_action_id or derive from title
        let category = issue
            .repair_action_id
            .as_ref()
            .map(|id| id.replace('-', "_"))
            .unwrap_or_else(|| {
                // Fallback: derive from title
                issue
                    .title
                    .to_lowercase()
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join("_")
            });

        Self {
            key: issue.issue_key(),
            title: issue.title.clone(),
            severity: severity.to_string(),
            visibility: visibility.to_string(),
            category,
            summary: issue.explanation.clone(),
            recommended_action: Some(issue.recommended_action.clone()),
            repair_action_id: issue.repair_action_id.clone(),
            reference: issue.reference.clone(),
            impact: issue.estimated_impact.clone(),
            decision,
        }
    }
}

/// Convert MachineProfile to string
pub fn profile_to_string(profile: MachineProfile) -> String {
    match profile {
        MachineProfile::Laptop => "Laptop".to_string(),
        MachineProfile::Desktop => "Desktop".to_string(),
        MachineProfile::ServerLike => "Server-Like".to_string(),
        MachineProfile::Unknown => "Unknown".to_string(),
    }
}

// Phase 5.3: Insights JSON output types

/// JSON output for insights command
#[derive(Debug, Clone, Serialize)]
pub struct InsightsJson {
    /// JSON schema version for stable scripting
    pub schema_version: String,
    /// Timestamp when insights were generated
    pub generated_at: String,
    /// Machine profile at time of generation
    pub profile: String,
    /// Analysis window in days
    pub analysis_window_days: i64,
    /// Total observations analyzed
    pub total_observations: usize,
    /// Flapping issues (appear/disappear repeatedly)
    pub flapping: Vec<FlappingIssueJson>,
    /// Escalating issues (severity increases over time)
    pub escalating: Vec<EscalatingIssueJson>,
    /// Long-term unaddressed issues (visible >14 days)
    pub long_term: Vec<LongTermIssueJson>,
    /// Profile transitions detected
    pub profile_transitions: Vec<ProfileTransitionJson>,
    /// Top recurring issues by appearance count
    pub top_recurring_issues: Vec<RecurringIssueJson>,
}

/// Flapping issue JSON representation
#[derive(Debug, Clone, Serialize)]
pub struct FlappingIssueJson {
    /// Stable issue key
    pub issue_key: String,
    /// Human-readable description
    pub description: String,
    /// Number of visibility state changes
    pub state_changes: usize,
    /// Days observed
    pub days_observed: i64,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
}

/// Escalating issue JSON representation
#[derive(Debug, Clone, Serialize)]
pub struct EscalatingIssueJson {
    /// Stable issue key
    pub issue_key: String,
    /// Human-readable description
    pub description: String,
    /// Severity progression details
    pub severity_path: String,
    /// Days observed
    pub days_observed: i64,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
}

/// Long-term issue JSON representation
#[derive(Debug, Clone, Serialize)]
pub struct LongTermIssueJson {
    /// Stable issue key
    pub issue_key: String,
    /// Human-readable description
    pub description: String,
    /// Days visible without resolution
    pub days_visible: i64,
    /// Number of observations
    pub observation_count: usize,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
}

/// Profile transition JSON representation
#[derive(Debug, Clone, Serialize)]
pub struct ProfileTransitionJson {
    /// Stable issue key
    pub issue_key: String,
    /// Human-readable description
    pub description: String,
    /// Transition details (e.g., "Laptop → Desktop")
    pub transition_details: String,
    /// Days observed
    pub days_observed: i64,
}

/// Recurring issue JSON representation
#[derive(Debug, Clone, Serialize)]
pub struct RecurringIssueJson {
    /// Stable issue key
    pub issue_key: String,
    /// Number of times issue appeared
    pub appearance_count: usize,
    /// First seen timestamp (RFC3339)
    pub first_seen: String,
    /// Last seen timestamp (RFC3339)
    pub last_seen: String,
}

// Phase 5.4: Weekly summary JSON output types

/// JSON output for weekly command
#[derive(Debug, Clone, Serialize)]
pub struct WeeklyJson {
    /// JSON schema version for stable scripting
    pub schema_version: String,
    /// Timestamp when report was generated
    pub generated_at: String,
    /// Machine profile at time of generation
    pub profile: String,
    /// Window start timestamp (RFC3339)
    pub window_start: String,
    /// Window end timestamp (RFC3339)
    pub window_end: String,
    /// Total observations analyzed
    pub total_observations: u64,
    /// Recurring issues (flapping)
    pub recurring_issues: Vec<RecurringIssueJson>,
    /// Escalating issues
    pub escalating_issues: Vec<EscalatingIssueJson>,
    /// Long-term unaddressed issues
    pub long_term_issues: Vec<LongTermIssueJson>,
    /// Repairs executed this week
    pub repairs: Vec<WeeklyRepairJson>,
    /// Rule-based habit suggestions
    pub suggestions: Vec<String>,
}

/// Weekly repair summary JSON representation
#[derive(Debug, Clone, Serialize)]
pub struct WeeklyRepairJson {
    /// Repair action identifier
    pub repair_action_id: String,
    /// Number of times this repair ran
    pub times_ran: u32,
    /// Last run timestamp (RFC3339)
    pub last_run: Option<String>,
}
