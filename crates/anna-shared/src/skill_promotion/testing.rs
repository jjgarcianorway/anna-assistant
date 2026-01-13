//! Recipe Candidate Testing - Validate candidates before promotion.
//!
//! Tests run in increasing fidelity levels:
//! 1. Static analysis (precondition checks)
//! 2. Dry-run simulation
//! 3. Sandbox execution (future: namespaces, containers, VMs)
//!
//! v0.3.14: Initial implementation with static checks

use super::candidate::{Precondition, PreconditionType, RecipeCandidate, RiskLevel};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Result of testing a recipe candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Overall status
    pub status: TestStatus,
    /// Individual test results
    pub checks: Vec<TestCheck>,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl TestResult {
    pub fn passed(&self) -> bool {
        matches!(self.status, TestStatus::Passed | TestStatus::PassedWithWarnings)
    }
}

/// Overall test status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    /// All checks passed
    Passed,
    /// Passed but with warnings
    PassedWithWarnings,
    /// Failed one or more checks
    Failed,
    /// Could not complete testing
    Error,
}

/// Individual test check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCheck {
    /// Name of the check
    pub name: String,
    /// Whether it passed
    pub passed: bool,
    /// Details or error message
    pub message: String,
    /// Severity if failed
    pub severity: CheckSeverity,
}

/// Severity of a failed check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Risk assessment for a candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Calculated risk level
    pub level: RiskLevel,
    /// Factors contributing to risk
    pub factors: Vec<String>,
    /// Whether rollback is available
    pub has_rollback: bool,
    /// Confidence in rollback
    pub rollback_confidence: f32,
}

/// Test a recipe candidate
pub fn test_candidate(candidate: &RecipeCandidate) -> TestResult {
    let mut checks = Vec::new();
    let mut recommendations = Vec::new();

    // 1. Check preconditions can be validated
    for precondition in &candidate.preconditions {
        let check = test_precondition(precondition);
        checks.push(check);
    }

    // 2. Validate command safety
    for (i, cmd) in candidate.commands.iter().enumerate() {
        let check = validate_command(&cmd.command, i);
        if !check.passed {
            recommendations.push(format!(
                "Command {} may need review: {}",
                i + 1,
                check.message
            ));
        }
        checks.push(check);
    }

    // 3. Check rollback coverage
    let rollback_check = validate_rollback(candidate);
    if !rollback_check.passed {
        recommendations.push("Consider adding rollback steps for all modifying commands".to_string());
    }
    checks.push(rollback_check);

    // 4. Risk assessment
    let risk_assessment = assess_risk(candidate);
    if risk_assessment.level as u8 >= RiskLevel::High as u8 {
        recommendations.push("High-risk recipe - consider requiring explicit confirmation".to_string());
    }

    // 5. Determine overall status
    let has_critical = checks.iter().any(|c| !c.passed && c.severity == CheckSeverity::Critical);
    let has_error = checks.iter().any(|c| !c.passed && c.severity == CheckSeverity::Error);
    let has_warning = checks.iter().any(|c| !c.passed && c.severity == CheckSeverity::Warning);

    let status = if has_critical || has_error {
        TestStatus::Failed
    } else if has_warning {
        TestStatus::PassedWithWarnings
    } else {
        TestStatus::Passed
    };

    TestResult {
        status,
        checks,
        risk_assessment,
        recommendations,
    }
}

/// Test a single precondition
fn test_precondition(precondition: &Precondition) -> TestCheck {
    match &precondition.condition_type {
        PreconditionType::CommandExists(cmd) => {
            let exists = Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            TestCheck {
                name: format!("precondition:command:{}", cmd),
                passed: exists,
                message: if exists {
                    format!("Command '{}' is available", cmd)
                } else {
                    format!("Command '{}' not found - will need to be installed", cmd)
                },
                severity: CheckSeverity::Warning, // Not fatal, might be installed
            }
        }
        PreconditionType::FileExists(path) => {
            let exists = std::path::Path::new(path).exists();
            TestCheck {
                name: format!("precondition:file:{}", path),
                passed: exists,
                message: if exists {
                    format!("File '{}' exists", path)
                } else {
                    format!("File '{}' not found", path)
                },
                severity: CheckSeverity::Warning,
            }
        }
        PreconditionType::PackageInstalled(pkg) => {
            let installed = Command::new("pacman")
                .args(["-Qi", pkg])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            TestCheck {
                name: format!("precondition:package:{}", pkg),
                passed: installed,
                message: if installed {
                    format!("Package '{}' is installed", pkg)
                } else {
                    format!("Package '{}' not installed", pkg)
                },
                severity: CheckSeverity::Warning,
            }
        }
        PreconditionType::ServiceRunning(service) => {
            let running = Command::new("systemctl")
                .args(["is-active", "--quiet", service])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            TestCheck {
                name: format!("precondition:service:{}", service),
                passed: running,
                message: if running {
                    format!("Service '{}' is running", service)
                } else {
                    format!("Service '{}' is not running", service)
                },
                severity: CheckSeverity::Info,
            }
        }
        _ => TestCheck {
            name: "precondition:custom".to_string(),
            passed: true,
            message: "Custom precondition - skipped".to_string(),
            severity: CheckSeverity::Info,
        },
    }
}

/// Validate a command for safety issues
fn validate_command(cmd: &str, index: usize) -> TestCheck {
    let cmd_lower = cmd.to_lowercase();

    // Check for dangerous patterns
    let dangerous_patterns = [
        ("rm -rf /", "Recursive delete of root filesystem"),
        ("dd if=", "Direct disk write - could destroy data"),
        ("> /dev/sd", "Direct write to block device"),
        ("chmod 777", "World-writable permissions - security risk"),
        ("curl | bash", "Piped remote execution - security risk"),
        ("wget | sh", "Piped remote execution - security risk"),
        (":(){:|:&};:", "Fork bomb"),
        ("mkfs", "Filesystem creation - destroys existing data"),
    ];

    for (pattern, reason) in dangerous_patterns {
        if cmd_lower.contains(pattern) {
            return TestCheck {
                name: format!("command:safety:{}", index),
                passed: false,
                message: format!("Dangerous command pattern: {}", reason),
                severity: CheckSeverity::Critical,
            };
        }
    }

    // Check for potentially risky patterns
    let risky_patterns = [
        ("--force", "Force flag may bypass safety checks"),
        ("--noconfirm", "Skips confirmation prompts"),
        ("-y", "May auto-confirm operations"),
        ("rm -r", "Recursive delete"),
        ("sudo", "Elevated privileges"),
    ];

    for (pattern, reason) in risky_patterns {
        if cmd_lower.contains(pattern) {
            return TestCheck {
                name: format!("command:risk:{}", index),
                passed: true, // Not a failure, just a warning
                message: format!("Note: {}", reason),
                severity: CheckSeverity::Info,
            };
        }
    }

    TestCheck {
        name: format!("command:safety:{}", index),
        passed: true,
        message: "Command passed safety checks".to_string(),
        severity: CheckSeverity::Info,
    }
}

/// Validate rollback coverage
fn validate_rollback(candidate: &RecipeCandidate) -> TestCheck {
    let modifying_commands = candidate
        .commands
        .iter()
        .filter(|c| c.modifies_system)
        .count();

    let rollback_steps = candidate.rollback.len();

    if modifying_commands == 0 {
        return TestCheck {
            name: "rollback:coverage".to_string(),
            passed: true,
            message: "No modifying commands - rollback not needed".to_string(),
            severity: CheckSeverity::Info,
        };
    }

    let coverage = rollback_steps as f32 / modifying_commands as f32;

    TestCheck {
        name: "rollback:coverage".to_string(),
        passed: coverage >= 0.5, // At least 50% coverage
        message: format!(
            "Rollback coverage: {:.0}% ({}/{} commands covered)",
            coverage * 100.0,
            rollback_steps,
            modifying_commands
        ),
        severity: if coverage >= 0.8 {
            CheckSeverity::Info
        } else if coverage >= 0.5 {
            CheckSeverity::Warning
        } else {
            CheckSeverity::Error
        },
    }
}

/// Assess the risk of a candidate
fn assess_risk(candidate: &RecipeCandidate) -> RiskAssessment {
    let mut factors = Vec::new();

    // Count risk factors
    let sudo_count = candidate
        .commands
        .iter()
        .filter(|c| c.needs_root)
        .count();

    let modify_count = candidate
        .commands
        .iter()
        .filter(|c| c.modifies_system)
        .count();

    if sudo_count > 0 {
        factors.push(format!("{} commands require root", sudo_count));
    }

    if modify_count > 0 {
        factors.push(format!("{} commands modify system", modify_count));
    }

    // Check rollback confidence
    let has_rollback = !candidate.rollback.is_empty();
    let rollback_confidence = if has_rollback {
        let coverage = candidate.rollback.len() as f32 / modify_count.max(1) as f32;
        coverage.min(1.0)
    } else {
        0.0
    };

    if !has_rollback && modify_count > 0 {
        factors.push("No rollback steps defined".to_string());
    }

    RiskAssessment {
        level: candidate.risk_level,
        factors,
        has_rollback,
        rollback_confidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::ExperienceContext;
    use crate::skill_promotion::generate_candidate;
    use crate::memory::Experience;

    fn make_test_experience() -> Experience {
        Experience {
            id: "test-123".to_string(),
            question: "how to list files".to_string(),
            keywords: vec!["list".to_string(), "files".to_string()],
            successful_commands: vec!["ls -la".to_string()],
            answer: "Use ls -la to list files".to_string(),
            context: ExperienceContext::default(),
            usefulness_score: 5,
            created_at: "2024-01-01".to_string(),
            last_used: None,
            embedding: None,
        }
    }

    #[test]
    fn test_safe_candidate_passes() {
        let exp = make_test_experience();
        let candidate = generate_candidate(&exp);
        let result = test_candidate(&candidate);
        assert!(result.passed());
    }

    #[test]
    fn test_dangerous_command_fails() {
        let check = validate_command("rm -rf /", 0);
        assert!(!check.passed);
        assert_eq!(check.severity, CheckSeverity::Critical);
    }
}
