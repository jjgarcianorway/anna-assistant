//! Core types for the Assisted Operations Layer.
//!
//! These are pure data structures. They describe what COULD be done.
//! They do not execute anything.

use serde::{Deserialize, Serialize};

/// An assisted operation that Anna has prepared for human review.
///
/// # Purpose
///
/// This structure contains everything a human needs to understand a problem
/// and decide whether to apply a proposed fix. Anna prepares this information
/// but NEVER executes the commands herself.
///
/// # Fields
///
/// - `operation_id`: Unique identifier for tracking
/// - `detected_problem`: What Anna found wrong
/// - `explanation`: Why this is a problem and how the fix works
/// - `proposed_steps`: Ordered list of commands for the human to run
/// - `risk_level`: How dangerous these changes are
/// - `sources`: Citations (Arch Wiki, man pages, etc.)
/// - `requires_reboot`: Whether changes need a reboot to take effect
///
/// # Execution Model
///
/// 1. Anna creates this structure
/// 2. Anna presents it to the human
/// 3. Human reviews each proposed step
/// 4. Human runs commands manually (copy/paste to terminal)
/// 5. Human confirms each step is complete
/// 6. Anna re-checks system state
/// 7. Repeat until done
///
/// Anna NEVER runs the commands. The human ALWAYS runs them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistedOperation {
    /// Unique identifier for this operation
    pub operation_id: String,

    /// What problem was detected
    pub detected_problem: String,

    /// Human-readable explanation of the problem and proposed fix
    pub explanation: String,

    /// Ordered list of steps for the human to execute
    pub proposed_steps: Vec<ProposedStep>,

    /// How risky are these changes
    pub risk_level: RiskLevel,

    /// Sources of information (Arch Wiki URLs, man pages, etc.)
    pub sources: Vec<Source>,

    /// Whether a reboot is required after all steps
    pub requires_reboot: bool,

    /// Diagnosis summary - what was observed (Phase 43)
    #[serde(default)]
    pub diagnosis_summary: String,
}

impl AssistedOperation {
    /// Get commands that are safe to run automatically (Phase 43).
    /// These are in the HumanExecutionAdapter allowlist.
    pub fn safe_commands(&self) -> Vec<&ProposedStep> {
        self.proposed_steps
            .iter()
            .filter(|s| s.safety == CommandSafety::SafeAutomatic)
            .collect()
    }

    /// Get commands that must be run manually (Phase 43).
    /// These require sudo or have risks.
    pub fn manual_commands(&self) -> Vec<&ProposedStep> {
        self.proposed_steps
            .iter()
            .filter(|s| s.safety == CommandSafety::ManualOnly)
            .collect()
    }

    /// Get all citations as plain strings (Phase 43).
    pub fn citation_urls(&self) -> Vec<&str> {
        self.sources.iter().map(|s| s.reference.as_str()).collect()
    }
}

/// A single proposed step for the human to execute.
///
/// Anna prepares this. The human runs it. Anna cannot run it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedStep {
    /// Step number in sequence
    pub step_number: u32,

    /// Human-readable description of what this step does
    pub description: String,

    /// The exact command to run (verbatim, for copy/paste)
    pub exact_command: String,

    /// Why this step is necessary
    pub why: String,

    /// Whether this step can be undone
    pub reversible: bool,

    /// Command to reverse this step (if reversible)
    pub reverse_command: Option<String>,

    /// Execution safety classification (Phase 43)
    #[serde(default)]
    pub safety: CommandSafety,
}

/// Safety classification for a proposed command (Phase 43).
///
/// This determines whether Anna can execute the command automatically
/// through HumanExecutionAdapter, or if it must be copy/pasted manually.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommandSafety {
    /// Safe to run automatically - in HumanExecutionAdapter allowlist
    /// No sudo, no pipes, no redirects, no dangerous operations
    SafeAutomatic,
    /// Must be run manually by human - requires sudo or has risks
    /// Anna will display as copy/paste instructions
    #[default]
    ManualOnly,
}

/// Risk level for an assisted operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Changes are purely diagnostic or configuration
    Low,
    /// Changes affect system behavior but are reversible
    Medium,
    /// Changes may cause issues if done incorrectly
    High,
    /// Changes could break the system
    Critical,
}

impl RiskLevel {
    /// Human-readable description of the risk level.
    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low risk - safe to try, easily reversible",
            RiskLevel::Medium => "Medium risk - reversible but may require attention",
            RiskLevel::High => "High risk - could cause issues if done incorrectly",
            RiskLevel::Critical => "Critical risk - could break the system, proceed with caution",
        }
    }
}

/// A source of information used to prepare the operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Type of source
    pub source_type: SourceType,

    /// Title or description
    pub title: String,

    /// URL or reference
    pub reference: String,
}

/// Type of information source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Arch Wiki article
    ArchWiki,
    /// Man page
    ManPage,
    /// Upstream documentation
    Upstream,
    /// Kernel documentation
    Kernel,
    /// Forum or community post
    Community,
}

/// Status of an assisted operation from the human's perspective.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    /// Anna has prepared the operation, awaiting human review
    Proposed,
    /// Human is reviewing the steps
    UnderReview,
    /// Human is executing steps (tracking which ones are done)
    InProgress { completed_steps: Vec<u32> },
    /// Human has completed all steps, Anna is verifying
    Verifying,
    /// Operation completed successfully
    Completed,
    /// Human chose not to proceed
    Declined,
    /// Something went wrong during execution
    Failed { reason: String },
}

/// A detected issue that could become an assisted operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedIssue {
    /// What was detected
    pub problem: String,

    /// Severity of the issue
    pub severity: IssueSeverity,

    /// Evidence gathered (command outputs, file contents, etc.)
    pub evidence: Vec<Evidence>,

    /// Can Anna propose a fix?
    pub fixable: bool,
}

/// Severity of a detected issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Informational, not necessarily a problem
    Info,
    /// Minor issue, system works but not optimally
    Minor,
    /// Significant issue affecting functionality
    Major,
    /// Critical issue requiring immediate attention
    Critical,
}

/// Evidence gathered during detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// What command or file was checked
    pub source: String,

    /// What was found
    pub finding: String,

    /// Raw output (for debugging)
    pub raw_output: Option<String>,
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES
// =============================================================================
//
// These types:
// - CANNOT execute commands (they are data, not functions)
// - CANNOT call std::process::Command
// - CANNOT interact with the system beyond being printed/displayed
// - CANNOT bypass any confirmation step
// - CANNOT auto-apply themselves
//
// The exact_command field is a String. It is displayed to humans.
// There is no code path that passes this string to a shell for execution.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_descriptions() {
        assert!(!RiskLevel::Low.description().is_empty());
        assert!(!RiskLevel::Medium.description().is_empty());
        assert!(!RiskLevel::High.description().is_empty());
        assert!(!RiskLevel::Critical.description().is_empty());
    }

    #[test]
    fn test_proposed_step_is_just_data() {
        let step = ProposedStep {
            step_number: 1,
            description: "Remove broken config".to_string(),
            exact_command: "sudo rm /etc/broken.conf".to_string(),
            why: "The config file is corrupted".to_string(),
            reversible: false,
            reverse_command: None,
            safety: CommandSafety::ManualOnly,
        };

        // The step is data. It has no execute() method.
        // It can only be displayed to a human.
        assert_eq!(step.exact_command, "sudo rm /etc/broken.conf");

        // There is no:
        // step.execute()
        // step.run()
        // step.apply()
    }

    #[test]
    fn test_operation_serialization() {
        let op = AssistedOperation {
            operation_id: "test-001".to_string(),
            detected_problem: "Test problem".to_string(),
            explanation: "Test explanation".to_string(),
            proposed_steps: vec![],
            risk_level: RiskLevel::Low,
            sources: vec![],
            requires_reboot: false,
            diagnosis_summary: String::new(),
        };

        // Can serialize to JSON for display
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("test-001"));

        // Can deserialize back
        let restored: AssistedOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, restored);
    }
}
