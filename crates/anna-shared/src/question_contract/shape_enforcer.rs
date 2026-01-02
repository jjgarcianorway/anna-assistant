//! Shape enforcement and violation detection - v0.0.437.
//!
//! Enforces answer shape constraints and detects violations.

use super::answer_field::AnswerField;
use super::answer_plan::AnswerPlan;
use super::intent::{QuestionIntent, Subject};

/// Enforces answer shape strictly.
pub struct ShapeEnforcer;

impl ShapeEnforcer {
    /// Filter specialist output to match intent.
    pub fn enforce(intent: &QuestionIntent, raw_fields: Vec<AnswerField>) -> EnforcementResult {
        let mut plan = AnswerPlan::new(intent);

        for field in raw_fields {
            plan.add_field(field);
        }

        let is_valid = plan.is_complete() || !plan.fields.is_empty();

        EnforcementResult {
            plan,
            violations: Vec::new(),
            is_valid,
        }
    }

    /// Check if a raw answer violates constraints.
    pub fn check_violations(intent: &QuestionIntent, raw_text: &str) -> Vec<ShapeViolation> {
        let mut violations = Vec::new();

        // Check for tutorial leakage in fact/status questions
        if !intent.category.allows_tutorials() {
            if raw_text.contains("you can") || raw_text.contains("try running") {
                violations.push(ShapeViolation::TutorialLeakage);
            }
            if raw_text.contains("```") {
                violations.push(ShapeViolation::CommandLeakage);
            }
        }

        // Check for off-topic content
        let subject_keywords = get_subject_keywords(intent.subject);
        let has_subject_content = subject_keywords
            .iter()
            .any(|k| raw_text.to_lowercase().contains(k));

        if !has_subject_content && !intent.allows_extras() {
            violations.push(ShapeViolation::OffTopic);
        }

        violations
    }
}

/// Result of shape enforcement.
#[derive(Debug, Clone)]
pub struct EnforcementResult {
    /// The filtered answer plan.
    pub plan: AnswerPlan,
    /// Violations found.
    pub violations: Vec<ShapeViolation>,
    /// Whether the result is valid.
    pub is_valid: bool,
}

/// Type of shape violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeViolation {
    /// Tutorial text in fact/status answer.
    TutorialLeakage,
    /// Command suggestions in fact/status answer.
    CommandLeakage,
    /// Content about wrong subject.
    OffTopic,
    /// Too many items.
    TooManyItems,
    /// Missing required fields.
    MissingFields,
}

/// Get keywords for a subject.
fn get_subject_keywords(subject: Subject) -> Vec<&'static str> {
    match subject {
        Subject::Memory => vec!["memory", "ram", "swap", "free", "available", "cached"],
        Subject::Cpu => vec!["cpu", "processor", "core", "thread", "frequency", "ghz"],
        Subject::Disk => vec![
            "disk",
            "storage",
            "filesystem",
            "mount",
            "partition",
            "gb",
            "tb",
        ],
        Subject::Service => vec!["service", "systemd", "unit", "running", "active", "failed"],
        Subject::Network => vec![
            "network",
            "interface",
            "ip",
            "ethernet",
            "wifi",
            "connection",
        ],
        Subject::Gpu => vec![
            "gpu", "graphics", "nvidia", "amd", "intel", "driver", "video",
        ],
        Subject::Boot => vec!["boot", "startup", "init", "kernel", "systemd-analyze"],
        Subject::Audio => vec!["audio", "sound", "pulseaudio", "pipewire", "alsa", "volume"],
        Subject::Packages => vec!["package", "installed", "pacman", "apt", "dnf", "version"],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::question_contract::answer_field::AnswerValue;
    use crate::question_contract::intent::{IntentBuilder, IntentCategory};

    #[test]
    fn test_shape_enforcer_detects_tutorial_leakage() {
        let intent = IntentBuilder::new("int_003")
            .category(IntentCategory::Fact)
            .build();

        let raw_text = "You have 4GB free RAM. You can try running free -h for more details.";
        let violations = ShapeEnforcer::check_violations(&intent, raw_text);

        assert!(violations.contains(&ShapeViolation::TutorialLeakage));
    }
}
