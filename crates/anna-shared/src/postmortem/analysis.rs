//! Failure Analysis - Systematic root cause analysis.

use serde::{Deserialize, Serialize};

/// Complete failure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAnalysis {
    /// Primary cause of the failure
    pub primary_cause: FailureCause,
    /// Contributing factors
    pub contributing_factors: Vec<FailureCause>,
    /// Category of failure
    pub category: FailureCategory,
    /// What signals were missed
    pub missed_signals: Vec<String>,
    /// The earliest probe that would have caught this
    pub earliest_detection: Option<String>,
    /// Corrected plan
    pub corrected_plan: Vec<String>,
    /// Countermeasures to prevent recurrence
    pub countermeasures: Vec<Countermeasure>,
    /// Confidence in analysis
    pub confidence: f32,
}

/// A specific cause of failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCause {
    /// What went wrong
    pub description: String,
    /// What command or action caused it
    pub trigger: Option<String>,
    /// Error message or output
    pub error_output: Option<String>,
    /// System state that contributed
    pub system_state: Vec<String>,
}

/// Categories of failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureCategory {
    /// Wrong command used
    WrongCommand,
    /// Right command, wrong arguments
    WrongArguments,
    /// Missing dependency
    MissingDependency,
    /// Permission denied
    PermissionDenied,
    /// Resource not available
    ResourceUnavailable,
    /// Timeout
    Timeout,
    /// Parse error in output
    ParseError,
    /// Assumption violated
    AssumptionViolated,
    /// Race condition / timing
    TimingIssue,
    /// Unknown / needs investigation
    Unknown,
}

impl FailureCategory {
    /// Suggest countermeasures based on category
    pub fn default_countermeasures(&self) -> Vec<CountermeasureType> {
        match self {
            FailureCategory::WrongCommand => vec![
                CountermeasureType::AddNegativeExample,
                CountermeasureType::UpdatePatternMatching,
            ],
            FailureCategory::WrongArguments => vec![
                CountermeasureType::AddNegativeExample,
                CountermeasureType::ImproveParsing,
            ],
            FailureCategory::MissingDependency => vec![
                CountermeasureType::AddPrecondition,
                CountermeasureType::AutoInstallDependency,
            ],
            FailureCategory::PermissionDenied => vec![
                CountermeasureType::AddPrecondition,
                CountermeasureType::SuggestSudo,
            ],
            FailureCategory::ResourceUnavailable => vec![
                CountermeasureType::AddPrecondition,
                CountermeasureType::AddRetry,
            ],
            FailureCategory::Timeout => vec![
                CountermeasureType::IncreaseTimeout,
                CountermeasureType::AddProgressCheck,
            ],
            FailureCategory::ParseError => vec![
                CountermeasureType::ImproveParsing,
                CountermeasureType::UseJsonOutput,
            ],
            FailureCategory::AssumptionViolated => vec![
                CountermeasureType::AddPrecondition,
                CountermeasureType::AddNegativeExample,
            ],
            FailureCategory::TimingIssue => vec![
                CountermeasureType::AddDelay,
                CountermeasureType::AddRetry,
            ],
            FailureCategory::Unknown => vec![CountermeasureType::FlagForReview],
        }
    }
}

/// Types of countermeasures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CountermeasureType {
    /// Add this to negative memory
    AddNegativeExample,
    /// Add a precondition check
    AddPrecondition,
    /// Update pattern matching rules
    UpdatePatternMatching,
    /// Improve output parsing
    ImproveParsing,
    /// Use JSON output if available
    UseJsonOutput,
    /// Auto-install missing dependency
    AutoInstallDependency,
    /// Suggest using sudo
    SuggestSudo,
    /// Increase timeout
    IncreaseTimeout,
    /// Add progress checking
    AddProgressCheck,
    /// Add delay before action
    AddDelay,
    /// Add retry logic
    AddRetry,
    /// Flag for human review
    FlagForReview,
}

/// A countermeasure to prevent recurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Countermeasure {
    /// Type of countermeasure
    pub measure_type: CountermeasureType,
    /// Specific implementation
    pub implementation: String,
    /// Priority (higher = more important)
    pub priority: u8,
}

/// Analyze a failure
pub fn analyze_failure(
    question: &str,
    commands: &[String],
    outputs: &[String],
    error_message: Option<&str>,
) -> FailureAnalysis {
    let category = categorize_failure(error_message, outputs);
    let primary_cause = identify_primary_cause(commands, outputs, error_message);
    let missed_signals = identify_missed_signals(outputs);
    let corrected_plan = generate_corrected_plan(question, commands, &category);
    let countermeasures = generate_countermeasures(&category, &primary_cause);

    FailureAnalysis {
        primary_cause,
        contributing_factors: Vec::new(), // TODO: deeper analysis
        category,
        missed_signals,
        earliest_detection: None, // TODO: trace back
        corrected_plan,
        countermeasures,
        confidence: 0.6, // Conservative default
    }
}

/// Categorize the failure
fn categorize_failure(error_message: Option<&str>, outputs: &[String]) -> FailureCategory {
    let error = error_message.unwrap_or("");
    let all_output = outputs.join("\n").to_lowercase();

    if error.contains("command not found") || all_output.contains("command not found") {
        return FailureCategory::MissingDependency;
    }

    if error.contains("permission denied") || all_output.contains("permission denied") {
        return FailureCategory::PermissionDenied;
    }

    if error.contains("timed out") || error.contains("timeout") {
        return FailureCategory::Timeout;
    }

    if error.contains("no such file") || all_output.contains("no such file") {
        return FailureCategory::ResourceUnavailable;
    }

    if error.contains("parse") || error.contains("json") || error.contains("invalid") {
        return FailureCategory::ParseError;
    }

    if error.contains("race") || error.contains("concurrent") || error.contains("busy") {
        return FailureCategory::TimingIssue;
    }

    FailureCategory::Unknown
}

/// Identify the primary cause
fn identify_primary_cause(
    commands: &[String],
    outputs: &[String],
    error_message: Option<&str>,
) -> FailureCause {
    let trigger = commands.last().map(|s| s.to_string());
    let error_output = error_message.map(|s| s.to_string());

    let description = if let Some(err) = error_message {
        format!("Command failed: {}", err)
    } else {
        "Command produced unexpected output".to_string()
    };

    FailureCause {
        description,
        trigger,
        error_output,
        system_state: Vec::new(), // TODO: capture system state
    }
}

/// Identify signals that were missed
fn identify_missed_signals(outputs: &[String]) -> Vec<String> {
    let mut signals = Vec::new();

    for output in outputs {
        if output.contains("warning") {
            signals.push("Warning in output was ignored".to_string());
        }
        if output.contains("deprecated") {
            signals.push("Deprecated feature was used".to_string());
        }
        if output.contains("not found") && !output.contains("command not found") {
            signals.push("Resource not found signal".to_string());
        }
    }

    signals
}

/// Generate a corrected plan
fn generate_corrected_plan(
    _question: &str,
    commands: &[String],
    category: &FailureCategory,
) -> Vec<String> {
    let mut plan = Vec::new();

    match category {
        FailureCategory::MissingDependency => {
            plan.push("1. Check if required command exists".to_string());
            plan.push("2. Install missing dependency if needed".to_string());
            plan.push("3. Retry original command".to_string());
        }
        FailureCategory::PermissionDenied => {
            plan.push("1. Check if elevated privileges are needed".to_string());
            plan.push("2. Use sudo if appropriate".to_string());
            plan.push("3. Retry with proper permissions".to_string());
        }
        FailureCategory::Timeout => {
            plan.push("1. Check if service is responsive".to_string());
            plan.push("2. Increase timeout if needed".to_string());
            plan.push("3. Consider async approach".to_string());
        }
        _ => {
            plan.push("1. Review original command".to_string());
            if let Some(cmd) = commands.last() {
                plan.push(format!("2. Investigate: {}", cmd));
            }
            plan.push("3. Try alternative approach".to_string());
        }
    }

    plan
}

/// Generate countermeasures
fn generate_countermeasures(
    category: &FailureCategory,
    cause: &FailureCause,
) -> Vec<Countermeasure> {
    let mut measures = Vec::new();

    for measure_type in category.default_countermeasures() {
        let implementation = match measure_type {
            CountermeasureType::AddNegativeExample => {
                format!("Store failure: {}", cause.description)
            }
            CountermeasureType::AddPrecondition => {
                "Add check before executing command".to_string()
            }
            CountermeasureType::UseJsonOutput => {
                "Switch to JSON output for better parsing".to_string()
            }
            _ => format!("{:?}", measure_type),
        };

        measures.push(Countermeasure {
            measure_type,
            implementation,
            priority: match measure_type {
                CountermeasureType::AddNegativeExample => 10,
                CountermeasureType::AddPrecondition => 8,
                _ => 5,
            },
        });
    }

    // Sort by priority
    measures.sort_by(|a, b| b.priority.cmp(&a.priority));
    measures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_failure() {
        assert_eq!(
            categorize_failure(Some("command not found: foo"), &[]),
            FailureCategory::MissingDependency
        );

        assert_eq!(
            categorize_failure(Some("permission denied"), &[]),
            FailureCategory::PermissionDenied
        );
    }

    #[test]
    fn test_analyze_failure() {
        let analysis = analyze_failure(
            "how to install vim",
            &["pacman -S vim".to_string()],
            &["error: permission denied".to_string()],
            Some("permission denied"),
        );

        assert_eq!(analysis.category, FailureCategory::PermissionDenied);
        assert!(!analysis.countermeasures.is_empty());
    }
}
